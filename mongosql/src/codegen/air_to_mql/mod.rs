use crate::air::{self, MQLOperator};
use bson::{bson, doc, Bson};
use thiserror::Error;

#[cfg(test)]
mod test;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("air method is not implemented")]
    UnimplementedAIR,
}

#[derive(PartialEq, Debug)]
pub struct MqlTranslation {
    pub database: Option<String>,
    pub collection: Option<String>,
    pub pipeline: Vec<bson::Document>,
}

#[derive(Clone, Debug)]
pub struct MqlCodeGenerator {}

impl MqlCodeGenerator {
    fn to_mql_op(mqlo: MQLOperator) -> &'static str {
        use MQLOperator::*;
        match mqlo {
            // String operators
            Concat => "$concat",

            // Arithmetic operators
            Add => "$add",
            Subtract => "$subtract",
            Multiply => "$multiply",
            Divide => "$divide",

            // Comparison operators
            Lt => "$lt",
            Lte => "$lte",
            Ne => "$ne",
            Eq => "$eq",
            Gt => "$gt",
            Gte => "$gte",

            // Boolean operators
            Not => "$not",
            And => "$and",
            Or => "$or",

            // Array scalar functions
            Slice => "$slice",
            Size => "$size",

            // Numeric value scalar functions
            IndexOfCP => "$indexOfCP",
            IndexOfBytes => "$indexOfBytes",
            StrLenCP => "$strLenCP",
            StrLenBytes => "$strLenBytes",
            Abs => "$abs",
            Ceil => "$ceil",
            Cos => "$cos",
            DegreesToRadians => "$degreesToRadians",
            Floor => "$floor",
            Log => "$log",
            Mod => "$mod",
            Pow => "$pow",
            RadiansToDegrees => "$radiansToDegrees",
            Round => "$round",
            Sin => "$sin",
            Tan => "$tan",
            Sqrt => "$sqrt",

            // String value scalar functions
            SubstrCP => "$substrCP",
            SubstrBytes => "$substrBytes",
            ToUpper => "$toUpper",
            ToLower => "$toLower",
            Trim => "$trim",
            LTrim => "$ltrim",
            RTrim => "$rtrim",
            Split => "$split",

            // Datetime value scalar function
            Year => "$year",
            Month => "$month",
            DayOfMonth => "$dayOfMonth",
            Hour => "$hour",
            Minute => "$minute",
            Second => "$second",
            Week => "$week",
            DayOfYear => "$dayOfYear",
            IsoWeek => "$isoWeek",
            IsoDayOfWeek => "$isoDayOfWeek",
            DateAdd => "$dateAdd",
            DateDiff => "$dateDiff",
            DateTrunc => "$dateTrunc",

            // MergeObjects merges an array of objects
            MergeObjects => "$mergeObjects",
        }
    }

    pub fn codegen_air_expression(&self, expr: air::Expression) -> Result<bson::Bson> {
        use air::{Expression::*, LiteralValue::*};
        match expr {
            Literal(lit) => Ok(bson::bson!({
                "$literal": match lit {
                    Null => Bson::Null,
                    Boolean(b) => Bson::Boolean(b),
                    String(s) => Bson::String(s),
                    Integer(i) => Bson::Int32(i),
                    Long(l) => Bson::Int64(l),
                    Double(d) => Bson::Double(d),
                },
            })),
            Document(document) => Ok(Bson::Document({
                if document.is_empty() {
                    bson::doc! {"$literal": {}}
                } else {
                    document
                        .into_iter()
                        .map(|(k, v)| Ok((k, self.codegen_air_expression(v)?)))
                        .collect::<Result<bson::Document>>()?
                }
            })),
            Array(array) => Ok(Bson::Array(
                array
                    .into_iter()
                    .map(|e| self.codegen_air_expression(e))
                    .collect::<Result<Vec<Bson>>>()?,
            )),
            Variable(var) => Ok(Bson::String(format!("$${var}"))),
            FieldRef(fr) => Ok(Bson::String(self.codegen_field_ref(fr))),
            MQLSemanticOperator(mqls) => {
                let ops = mqls
                    .args
                    .into_iter()
                    .map(|x| self.codegen_air_expression(x))
                    .collect::<Result<Vec<_>>>()?;
                let operator = Self::to_mql_op(mqls.op);
                Ok(bson::bson!({ operator: Bson::Array(ops) }))
            }
            GetField(gf) => Ok({
                let input = self.codegen_air_expression(*gf.input)?;
                bson!({
                    "$getField": {
                        "field": gf.field,
                        "input": input,
                    }
                })
            }),
            _ => Err(Error::UnimplementedAIR),
        }
    }

    #[allow(clippy::only_used_in_recursion)] // false positive
    fn codegen_field_ref(&self, field_ref: air::FieldRef) -> String {
        match field_ref.parent {
            None => format!("${}", field_ref.name),
            Some(parent) => format!("{}.{}", self.codegen_field_ref(*parent), field_ref.name),
        }
    }

    pub fn codegen_air_stage(&self, stage: air::Stage) -> Result<MqlTranslation> {
        match stage {
            air::Stage::Project(p) => self.codegen_project(p),
            air::Stage::Group(_g) => Err(Error::UnimplementedAIR),
            air::Stage::Limit(_l) => Err(Error::UnimplementedAIR),
            air::Stage::Sort(_s) => Err(Error::UnimplementedAIR),
            air::Stage::Collection(c) => self.codegen_collection(c),
            air::Stage::Join(_j) => Err(Error::UnimplementedAIR),
            air::Stage::Unwind(_u) => Err(Error::UnimplementedAIR),
            air::Stage::Lookup(_l) => Err(Error::UnimplementedAIR),
            air::Stage::ReplaceWith(r) => self.codegen_replace_with(r),
            air::Stage::Match(_m) => Err(Error::UnimplementedAIR),
            air::Stage::UnionWith(_u) => Err(Error::UnimplementedAIR),
            air::Stage::Skip(_s) => Err(Error::UnimplementedAIR),
            air::Stage::Documents(d) => self.codegen_documents(d),
        }
    }

    fn codegen_replace_with(&self, air_replace_with: air::ReplaceWith) -> Result<MqlTranslation> {
        let source_translation = self.codegen_air_stage(*air_replace_with.source)?;
        let mut pipeline = source_translation.pipeline;
        let expr = self.codegen_air_expression(*air_replace_with.new_root)?;

        pipeline.push(doc! {"$replaceWith": expr});
        Ok(MqlTranslation {
            database: source_translation.database,
            collection: source_translation.collection,
            pipeline,
        })
    }

    fn codegen_documents(&self, air_docs: air::Documents) -> Result<MqlTranslation> {
        let docs = air_docs
            .array
            .into_iter()
            .map(|e| self.codegen_air_expression(e))
            .collect::<Result<Vec<Bson>>>()?;
        Ok(MqlTranslation {
            database: None,
            collection: None,
            pipeline: vec![doc! {"$documents": Bson::Array(docs)}],
        })
    }

    fn codegen_collection(&self, air_coll: air::Collection) -> Result<MqlTranslation> {
        Ok(MqlTranslation {
            database: Some(air_coll.db),
            collection: Some(air_coll.collection),
            pipeline: vec![],
        })
    }

    fn codegen_project(&self, air_project: air::Project) -> Result<MqlTranslation> {
        let source_translation = self.codegen_air_stage(*air_project.source)?;
        let mut pipeline = source_translation.pipeline;
        let mut project_doc = air_project
            .specifications
            .into_iter()
            .map(|(k, v)| Ok((k, self.codegen_air_expression(v)?)))
            .collect::<Result<bson::Document>>()?;
        if !project_doc.contains_key("_id") {
            // we create a temporary so that _id: 0 will always be the first element
            // in the doc. This does add another linear factor to the code, but makes
            // testing easier.
            let mut tmp_project_doc = doc! {"_id": 0};
            tmp_project_doc.extend(project_doc);
            project_doc = tmp_project_doc;
        }
        pipeline.push(doc! {"$project": project_doc});
        Ok(MqlTranslation {
            database: source_translation.database,
            collection: source_translation.collection,
            pipeline,
        })
    }
}
