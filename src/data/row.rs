use crate::data::cell::Cell;
use std::cmp::PartialOrd;
use std::hash::Hasher;
use crate::data::CellDefinition;
use crate::errors::JobError;

pub struct Row {
    pub cells: Vec<Cell>,
}

impl Row {
    pub fn concrete(mut self) -> Row {
        Row {cells: self.cells.drain(..).map(|c| c.concrete()).collect()}
    }

    pub fn partial_clone(&self) -> Result<Self, JobError> {
        Ok(Row {
            cells: self.cells.iter().map(|c| c.partial_clone()).collect::<Result<Vec<Cell>, JobError>>()?,
        })
    }
}

impl std::hash::Hash for Row {
    fn hash<H: Hasher>(&self, env: &mut H) {
        for c in &self.cells {
            c.hash(env);
        }
    }
}