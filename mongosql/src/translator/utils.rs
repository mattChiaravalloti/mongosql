use crate::{
    air::{self, LetVariable, MqlOperator, SqlOperator, TrimOperator},
    mapping_registry::{MqlMappingRegistry, MqlMappingRegistryValue, MqlReferenceType},
    mir::{self, ScalarFunction},
    translator::{Error, MqlTranslator, Result},
};
use mongosql_datastructures::binding_tuple::{BindingTuple, DatasourceName, Key};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarFunctionType {
    Divide,
    Mql(MqlOperator),
    Sql(SqlOperator),
    Trim(TrimOperator),
}

impl MqlTranslator {
    /// Generate a unique name given a predicate closure. Keeps pre-pending '_'
    /// until the predicate returns false, indicating that the name is not in
    /// use.
    pub(crate) fn generate_unique_datasource_name<F>(base_name: String, name_exists: F) -> String
    where
        F: Fn(&String) -> bool,
    {
        let mut ret = base_name;
        while name_exists(&ret) {
            ret.insert(0, '_')
        }
        ret
    }

    /// Ensures the base_name does not conflict with any datasource names at
    /// this scope. It does this by prepending '_' until there is no conflict.
    /// This function checks the project_names and self.mapping_registry for
    /// conflicts. This is because we need to handle conflicts within Projects,
    /// such as
    ///    SELECT `$foo`.*, `_foo`.* FROM ...
    /// as well as conflicts from datasources external to a Project, such as
    ///    SELECT * FROM `$foo` JOIN `_foo`
    ///
    /// In the first example, we will map `$foo` to `__foo` since we find
    /// `_foo` in the Project expression. We will map `_foo` to `_foo`.
    ///
    /// In the second example, we will map `$foo` to `_foo` since it has
    /// no conflicts in its individual (implicit) Project or in the mapping
    /// registry. We will map `_foo` to `__foo` since it will conflict with
    /// the `$foo` mapping from the mapping registry.
    pub(crate) fn ensure_unique_datasource_name(
        &mut self,
        base_name: String,
        project_names: &BindingTuple<mir::Expression>,
    ) -> String {
        MqlTranslator::generate_unique_datasource_name(base_name, |s| {
            let k = (s.clone(), self.scope_level).into();
            project_names.contains_key(&k)
                || (self.is_join && self.mapping_registry.contains_mapping(self.scope_level, s))
        })
    }

    pub(crate) fn get_datasource_name(
        datasource: &DatasourceName,
        unique_bot_name: &str,
    ) -> String {
        match datasource {
            DatasourceName::Bottom => unique_bot_name.to_string(),
            DatasourceName::Named(s) => s.clone(),
        }
    }

    /// Map a DatasourceName to a valid Mql project key name. Use the provided
    /// unique_bot_name for Bottom datasource. For datasources that contain '.'
    /// or start with '$', replace those characters with '_' and ensure the
    /// result is unique. This allows MongoSql to support datasource aliases
    /// that contain '.'s or start with '$'s. Project keys with those attributes
    /// have different semantics or are invalid in Mql, so we must replace them
    /// with valid characters.
    pub(crate) fn get_mapped_project_name(
        &mut self,
        datasource: &DatasourceName,
        unique_bot_name: &str,
        project_names: &BindingTuple<mir::Expression>,
    ) -> Result<String> {
        let mut mapped_k = Self::get_datasource_name(datasource, unique_bot_name);
        if mapped_k.as_str() == "" {
            return Err(Error::InvalidProjectField);
        }
        let mut needs_unique_check = false;
        if mapped_k.starts_with('$') {
            mapped_k = mapped_k.replacen('$', "_", 1);
            needs_unique_check = true;
        }
        if mapped_k.contains('.') {
            mapped_k = mapped_k.replace('.', "_");
            needs_unique_check = true;
        }
        needs_unique_check = needs_unique_check
            || (self.is_join
                && self
                    .mapping_registry
                    .contains_mapping(self.scope_level, &mapped_k));
        if needs_unique_check {
            mapped_k = self.ensure_unique_datasource_name(mapped_k, project_names);
        }
        Ok(mapped_k)
    }

    pub(crate) fn get_field_path_name(&self, fp: mir::FieldPath) -> Result<String> {
        let datasource_name = self
            .mapping_registry
            .get(&fp.key)
            .ok_or(Error::ReferenceNotFound(fp.key))
            .map(|s| s.name.clone())?;
        Ok(format!("{datasource_name}.{0}", fp.fields.join(".")))
    }

    pub(crate) fn generate_let_bindings(
        &mut self,
        registry: MqlMappingRegistry,
    ) -> Vec<LetVariable> {
        let mut let_bindings: Vec<LetVariable> = vec![];
        let new_mapping_registry = MqlMappingRegistry::with_registry(
            registry
                .get_registry()
                .clone()
                .into_iter()
                .map(|(key, value)| {
                    let mut generated_name = format!(
                        "v{}_{}",
                        Self::get_datasource_name(&key.datasource, "__bot"),
                        key.scope
                    );

                    // Here, we replace any invalid characters with an underscore
                    // and then lowercase the whole name. The [[:word:]] character
                    // class matches word characters ([0-9A-Za-z_]).
                    generated_name = regex::Regex::new(r"[[:ascii:]&&[:^word:]]")
                        .unwrap()
                        .replace_all(generated_name.as_str(), "_")
                        .to_string()
                        .to_ascii_lowercase();
                    while let_bindings.iter().any(|x| x.name == generated_name) {
                        generated_name.push('_');
                    }
                    let expr = match value.ref_type {
                        MqlReferenceType::FieldRef => air::Expression::FieldRef(value.name.into()),
                        MqlReferenceType::Variable => air::Expression::Variable(value.name.into()),
                    };
                    let_bindings.push(LetVariable {
                        name: generated_name.clone(),
                        expr: Box::new(expr),
                    });
                    (
                        key,
                        MqlMappingRegistryValue::new(generated_name, MqlReferenceType::Variable),
                    )
                })
                .collect::<BTreeMap<Key, MqlMappingRegistryValue>>(),
        );
        // update the mapping registry with the new values for existing keys
        self.mapping_registry.merge(new_mapping_registry);
        let_bindings
    }

    pub(crate) fn translate_literal_value(&self, lit: mir::LiteralValue) -> air::LiteralValue {
        match lit {
            mir::LiteralValue::Null => air::LiteralValue::Null,
            mir::LiteralValue::Boolean(b) => air::LiteralValue::Boolean(b),
            mir::LiteralValue::String(s) => air::LiteralValue::String(s),
            mir::LiteralValue::Integer(i) => air::LiteralValue::Integer(i),
            mir::LiteralValue::Long(l) => air::LiteralValue::Long(l),
            mir::LiteralValue::Double(d) => air::LiteralValue::Double(d),
            mir::LiteralValue::DbPointer(d) => air::LiteralValue::DbPointer(d),
            mir::LiteralValue::Undefined => air::LiteralValue::Undefined,
            mir::LiteralValue::DateTime(d) => air::LiteralValue::DateTime(d),
            mir::LiteralValue::Decimal128(d) => air::LiteralValue::Decimal128(d),
            mir::LiteralValue::MinKey => air::LiteralValue::MinKey,
            mir::LiteralValue::MaxKey => air::LiteralValue::MaxKey,
            mir::LiteralValue::Timestamp(t) => air::LiteralValue::Timestamp(t),
            mir::LiteralValue::RegularExpression(r) => air::LiteralValue::RegularExpression(r),
            mir::LiteralValue::ObjectId(o) => air::LiteralValue::ObjectId(o),
            mir::LiteralValue::JavaScriptCode(j) => air::LiteralValue::JavaScriptCode(j),
            mir::LiteralValue::JavaScriptCodeWithScope(j) => {
                air::LiteralValue::JavaScriptCodeWithScope(j)
            }
            mir::LiteralValue::Symbol(s) => air::LiteralValue::Symbol(s),
            mir::LiteralValue::Binary(b) => air::LiteralValue::Binary(b),
        }
    }
}

impl From<mir::Type> for air::Type {
    fn from(t: mir::Type) -> Self {
        match t {
            mir::Type::Array => air::Type::Array,
            mir::Type::BinData => air::Type::BinData,
            mir::Type::Boolean => air::Type::Boolean,
            mir::Type::Datetime => air::Type::Datetime,
            mir::Type::DbPointer => air::Type::DbPointer,
            mir::Type::Decimal128 => air::Type::Decimal128,
            mir::Type::Document => air::Type::Document,
            mir::Type::Double => air::Type::Double,
            mir::Type::Int32 => air::Type::Int32,
            mir::Type::Int64 => air::Type::Int64,
            mir::Type::Javascript => air::Type::Javascript,
            mir::Type::JavascriptWithScope => air::Type::JavascriptWithScope,
            mir::Type::MaxKey => air::Type::MaxKey,
            mir::Type::MinKey => air::Type::MinKey,
            mir::Type::Null => air::Type::Null,
            mir::Type::ObjectId => air::Type::ObjectId,
            mir::Type::RegularExpression => air::Type::RegularExpression,
            mir::Type::String => air::Type::String,
            mir::Type::Symbol => air::Type::Symbol,
            mir::Type::Timestamp => air::Type::Timestamp,
            mir::Type::Undefined => air::Type::Undefined,
        }
    }
}

impl From<mir::TypeOrMissing> for air::TypeOrMissing {
    fn from(item: mir::TypeOrMissing) -> Self {
        match item {
            mir::TypeOrMissing::Missing => air::TypeOrMissing::Missing,
            mir::TypeOrMissing::Type(t) => air::TypeOrMissing::Type(t.into()),
            mir::TypeOrMissing::Number => air::TypeOrMissing::Number,
        }
    }
}

impl From<mir::DatePart> for air::DatePart {
    fn from(dp: mir::DatePart) -> Self {
        match dp {
            mir::DatePart::Year => air::DatePart::Year,
            mir::DatePart::Quarter => air::DatePart::Quarter,
            mir::DatePart::Month => air::DatePart::Month,
            mir::DatePart::Week => air::DatePart::Week,
            mir::DatePart::Day => air::DatePart::Day,
            mir::DatePart::Hour => air::DatePart::Hour,
            mir::DatePart::Minute => air::DatePart::Minute,
            mir::DatePart::Second => air::DatePart::Second,
            mir::DatePart::Millisecond => air::DatePart::Millisecond,
        }
    }
}

impl From<mir::DateFunction> for air::DateFunction {
    fn from(df: mir::DateFunction) -> Self {
        match df {
            mir::DateFunction::Add => air::DateFunction::Add,
            mir::DateFunction::Diff => air::DateFunction::Diff,
            mir::DateFunction::Trunc => air::DateFunction::Trunc,
        }
    }
}

pub(crate) fn scalar_function_to_scalar_function_type(
    is_nullable: bool,
    function: mir::ScalarFunction,
) -> ScalarFunctionType {
    if is_nullable {
        ScalarFunctionType::from(function)
    } else {
        match function {
            ScalarFunction::Or => ScalarFunctionType::Mql(MqlOperator::Or),
            ScalarFunction::And => ScalarFunctionType::Mql(MqlOperator::And),
            ScalarFunction::Between => ScalarFunctionType::Mql(MqlOperator::Between),
            ScalarFunction::Eq => ScalarFunctionType::Mql(MqlOperator::Eq),
            ScalarFunction::Position => ScalarFunctionType::Mql(MqlOperator::IndexOfCP),
            ScalarFunction::Lt => ScalarFunctionType::Mql(MqlOperator::Lt),
            ScalarFunction::Lte => ScalarFunctionType::Mql(MqlOperator::Lte),
            ScalarFunction::Gt => ScalarFunctionType::Mql(MqlOperator::Gt),
            ScalarFunction::Gte => ScalarFunctionType::Mql(MqlOperator::Gte),
            ScalarFunction::Neq => ScalarFunctionType::Mql(MqlOperator::Ne),
            ScalarFunction::Not => ScalarFunctionType::Mql(MqlOperator::Not),
            ScalarFunction::Size => ScalarFunctionType::Mql(MqlOperator::Size),
            ScalarFunction::OctetLength => ScalarFunctionType::Mql(MqlOperator::StrLenBytes),
            ScalarFunction::CharLength => ScalarFunctionType::Mql(MqlOperator::StrLenCP),
            ScalarFunction::Substring => ScalarFunctionType::Mql(MqlOperator::SubstrCP),
            ScalarFunction::Lower => ScalarFunctionType::Mql(MqlOperator::ToLower),
            ScalarFunction::Upper => ScalarFunctionType::Mql(MqlOperator::ToUpper),
            _ => ScalarFunctionType::from(function),
        }
    }
}

impl From<mir::ScalarFunction> for ScalarFunctionType {
    fn from(func: mir::ScalarFunction) -> Self {
        use mir::ScalarFunction::*;
        match func {
            // String operators
            Concat => ScalarFunctionType::Mql(MqlOperator::Concat),

            // Unary arithmetic operators
            Pos => ScalarFunctionType::Sql(SqlOperator::Pos),
            Neg => ScalarFunctionType::Sql(SqlOperator::Neg),

            // Arithmetic operators
            Add => ScalarFunctionType::Mql(MqlOperator::Add),
            Sub => ScalarFunctionType::Mql(MqlOperator::Subtract),
            Mul => ScalarFunctionType::Mql(MqlOperator::Multiply),
            Div => ScalarFunctionType::Divide,

            // Comparison operators
            Lt => ScalarFunctionType::Sql(SqlOperator::Lt),
            Lte => ScalarFunctionType::Sql(SqlOperator::Lte),
            Neq => ScalarFunctionType::Sql(SqlOperator::Ne),
            Eq => ScalarFunctionType::Sql(SqlOperator::Eq),
            Gt => ScalarFunctionType::Sql(SqlOperator::Gt),
            Gte => ScalarFunctionType::Sql(SqlOperator::Gte),
            Between => ScalarFunctionType::Sql(SqlOperator::Between),

            // Boolean operators
            Not => ScalarFunctionType::Sql(SqlOperator::Not),
            And => ScalarFunctionType::Sql(SqlOperator::And),
            Or => ScalarFunctionType::Sql(SqlOperator::Or),

            // Computed Field Access operator
            // when the field is not known until runtime.
            ComputedFieldAccess => ScalarFunctionType::Sql(SqlOperator::ComputedFieldAccess),

            // Conditional scalar functions
            NullIf => ScalarFunctionType::Sql(SqlOperator::NullIf),
            Coalesce => ScalarFunctionType::Sql(SqlOperator::Coalesce),

            // Array scalar functions
            Slice => ScalarFunctionType::Sql(SqlOperator::Slice),
            Size => ScalarFunctionType::Sql(SqlOperator::Size),

            // Numeric value scalar functions
            Position => ScalarFunctionType::Sql(SqlOperator::IndexOfCP),
            CharLength => ScalarFunctionType::Sql(SqlOperator::StrLenCP),
            OctetLength => ScalarFunctionType::Sql(SqlOperator::StrLenBytes),
            BitLength => ScalarFunctionType::Sql(SqlOperator::BitLength),
            Abs => ScalarFunctionType::Mql(MqlOperator::Abs),
            Ceil => ScalarFunctionType::Mql(MqlOperator::Ceil),
            Cos => ScalarFunctionType::Sql(SqlOperator::Cos),
            Degrees => ScalarFunctionType::Mql(MqlOperator::RadiansToDegrees),
            Floor => ScalarFunctionType::Mql(MqlOperator::Floor),
            Log => ScalarFunctionType::Sql(SqlOperator::Log),
            Mod => ScalarFunctionType::Sql(SqlOperator::Mod),
            Pow => ScalarFunctionType::Mql(MqlOperator::Pow),
            Radians => ScalarFunctionType::Mql(MqlOperator::DegreesToRadians),
            Round => ScalarFunctionType::Sql(SqlOperator::Round),
            Sin => ScalarFunctionType::Sql(SqlOperator::Sin),
            Sqrt => ScalarFunctionType::Sql(SqlOperator::Sqrt),
            Tan => ScalarFunctionType::Sql(SqlOperator::Tan),

            // String value scalar functions
            Replace => ScalarFunctionType::Mql(MqlOperator::ReplaceAll),
            Substring => ScalarFunctionType::Sql(SqlOperator::SubstrCP),
            Upper => ScalarFunctionType::Sql(SqlOperator::ToUpper),
            Lower => ScalarFunctionType::Sql(SqlOperator::ToLower),
            BTrim => ScalarFunctionType::Trim(TrimOperator::Trim),
            LTrim => ScalarFunctionType::Trim(TrimOperator::LTrim),
            RTrim => ScalarFunctionType::Trim(TrimOperator::RTrim),
            Split => ScalarFunctionType::Sql(SqlOperator::Split),

            // Datetime value scalar function
            CurrentTimestamp => ScalarFunctionType::Sql(SqlOperator::CurrentTimestamp),
            Year => ScalarFunctionType::Mql(MqlOperator::Year),
            Month => ScalarFunctionType::Mql(MqlOperator::Month),
            Day => ScalarFunctionType::Mql(MqlOperator::DayOfMonth),
            Hour => ScalarFunctionType::Mql(MqlOperator::Hour),
            Minute => ScalarFunctionType::Mql(MqlOperator::Minute),
            Second => ScalarFunctionType::Mql(MqlOperator::Second),
            Millisecond => ScalarFunctionType::Mql(MqlOperator::Millisecond),
            Week => ScalarFunctionType::Mql(MqlOperator::Week),
            DayOfWeek => ScalarFunctionType::Mql(MqlOperator::DayOfWeek),
            DayOfYear => ScalarFunctionType::Mql(MqlOperator::DayOfYear),
            IsoWeek => ScalarFunctionType::Mql(MqlOperator::IsoWeek),
            IsoWeekday => ScalarFunctionType::Mql(MqlOperator::IsoDayOfWeek),

            // MergeObjects merges an array of objects
            MergeObjects => ScalarFunctionType::Mql(MqlOperator::MergeObjects),
        }
    }
}
