use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("malformed json: {0}")]
    MalformedJsonError(String),
    #[error("jmespath error: {0}")]
    Jmespath(#[from] jmespatch::JmespathError),
}

pub struct Query {
    expr: jmespatch::Expression<'static>,
}

impl Query {
    pub fn new(path: &str) -> Result<Self, Error> {
        let expr = jmespatch::compile(path)?;
        Ok(Self { expr })
    }

    pub fn extract_value(&self, json: &str) -> Result<Option<f64>, Error> {
        let var = jmespatch::Variable::from_json(&json).map_err(Error::MalformedJsonError)?;
        Ok(self.expr.search(var)?.as_number())
    }
}
