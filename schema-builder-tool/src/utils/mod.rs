use crate::Result;
use mongodb::{
    bson::{self, doc, Bson},
    Client,
};
use serde::{Deserialize, Serialize};

pub(crate) async fn check_cluster_type(client: &Client) -> Result<()> {
    match get_cluster_type(client).await? {
        ClusterType::AtlasDataFederation => Err(anyhow::Error::msg(
            "Atlas Data Federation is not supported. ADF schema is managed through Atlas.",
        )),
        ClusterType::Community => {
            Err(anyhow::Error::msg("Community clusters are not supported. Direct Cluster SQL Interface is only available for Enterprise users."))
        }
        ClusterType::Enterprise => Ok(()),
    }
}

async fn get_cluster_type(client: &Client) -> Result<ClusterType> {
    let cmd_res = client
        .database("admin")
        .run_command(doc! { "buildInfo": 1 })
        .await?;
    let deserializer = bson::Deserializer::new(Bson::Document(cmd_res));
    let deserializer = serde_stacker::Deserializer::new(deserializer);
    let bi: Result<BuildInfo> = Deserialize::deserialize(deserializer).map_err(|e| e.into());
    Ok(bi?.into())
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
enum ClusterType {
    AtlasDataFederation,
    Community,
    Enterprise,
}

impl From<BuildInfo> for ClusterType {
    fn from(build_info: BuildInfo) -> Self {
        if build_info.data_lake.is_some() {
            ClusterType::AtlasDataFederation
        } else if let Some(modules) = build_info.modules {
            if modules.contains(&"enterprise".to_string()) {
                ClusterType::Enterprise
            } else {
                ClusterType::Community
            }
        } else {
            ClusterType::Community
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BuildInfo {
    version: String,
    modules: Option<Vec<String>>,
    #[serde(rename = "dataLake")]
    data_lake: Option<DataLakeBuildInfo>,
    #[serde(rename = "versionArray")]
    version_array: Vec<i32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct DataLakeBuildInfo {
    pub version: String,
    #[serde(rename = "gitVersion")]
    pub git_version: String,
    pub date: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cluster_type_from_build_info() {
        let build_info = BuildInfo {
            version: "4.4.0".to_string(),
            modules: Some(vec!["enterprise".to_string()]),
            data_lake: None,
            version_array: vec![4, 4, 0],
        };
        assert_eq!(ClusterType::Enterprise, build_info.into());

        let build_info = BuildInfo {
            version: "4.4.0".to_string(),
            modules: None,
            data_lake: None,
            version_array: vec![4, 4, 0],
        };
        assert_eq!(ClusterType::Community, build_info.into());

        let build_info = BuildInfo {
            version: "4.4.0".to_string(),
            modules: None,
            data_lake: Some(DataLakeBuildInfo {
                version: "1.0.0".to_string(),
                git_version: "1.0.0".to_string(),
                date: "2020-01-01".to_string(),
            }),
            version_array: vec![4, 4, 0],
        };
        assert_eq!(ClusterType::AtlasDataFederation, build_info.into());
    }
}
