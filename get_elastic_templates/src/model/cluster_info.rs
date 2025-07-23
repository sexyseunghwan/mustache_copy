use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub hosts: Vec<String>,
    pub es_id: String,
    pub es_pw: String,
}
