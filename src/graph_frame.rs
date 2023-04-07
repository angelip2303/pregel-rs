use std::{error, fmt};
use std::fmt::{Debug, Display, Formatter};
use polars::prelude::*;

pub const ID: &str = "id";
pub const SRC: &str = "src";
pub const DST: &str = "dst";
pub const EDGE: &str = "edge";
pub const MSG: &str = "msg";

pub struct GraphFrame {
    pub vertices: LazyFrame,
    pub edges: LazyFrame
}

type Result<T> = std::result::Result<T, GraphFrameError>;

#[derive(Debug)]
pub enum GraphFrameError {
    FromPolars(PolarsError),
    MissingColumn(MissingColumnError)
}

impl Display for GraphFrameError {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GraphFrameError::FromPolars(error) => std::fmt::Display::fmt(error, f),
            GraphFrameError::MissingColumn(error) => std::fmt::Display::fmt(error, f),
        }
    }

}

impl error::Error for GraphFrameError {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            GraphFrameError::FromPolars(ref e) => Some(e),
            GraphFrameError::MissingColumn(_) => None,
        }
    }

}

#[derive(Debug)]
pub enum MissingColumnError {
    Id,
    Src,
    Dst
}

impl Display for MissingColumnError {

    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let message = |df, column|
            format!("The vertices {} must contain a {} for the Graph to be created", df, column);
        match self {
            MissingColumnError::Id =>  write!(f, "{}", message("vertices", ID)),
            MissingColumnError::Src => write!(f, "{}", message("edges", SRC)),
            MissingColumnError::Dst => write!(f, "{}", message("edges", DST)),
        }
    }

}

impl From<PolarsError> for GraphFrameError {
    fn from(err: PolarsError) -> GraphFrameError {
        GraphFrameError::FromPolars(err)
    }
}

impl GraphFrame {

    pub fn new(vertices: DataFrame, edges: DataFrame) -> Result<Self> {
        if !vertices.get_column_names().contains(&ID) {
            return Err(GraphFrameError::MissingColumn(MissingColumnError::Id));
        }
        if !edges.get_column_names().contains(&SRC) {
            return Err(GraphFrameError::MissingColumn(MissingColumnError::Src));
        }
        if !edges.get_column_names().contains(&DST) {
            return Err(GraphFrameError::MissingColumn(MissingColumnError::Dst));
        }

        Ok(
            GraphFrame {
                vertices: vertices.lazy(),
                edges: edges.lazy()
            }
        )
    }

    pub fn from_edges(edges: DataFrame) -> Result<Self> {
        let srcs = edges.clone().lazy().select([col(SRC).alias(ID)]);
        let dsts = edges.clone().lazy().select([col(DST).alias(ID)]);
        let vertices_lf = concat([srcs, dsts], false, true)?
            .unique(Some(vec!["id".to_string()]), UniqueKeepStrategy::First);
        let vertices = vertices_lf.collect()?;

        GraphFrame::new(vertices, edges.clone())
    }

    pub fn out_degrees(self) -> LazyFrame {
        self
            .edges
            .groupby([col(SRC).alias(ID)])
            .agg([count().alias("out_degree")])
    }

    pub fn in_degrees(self) -> LazyFrame {
        self
            .edges
            .groupby([col(DST)])
            .agg([count().alias("in_degree")])
    }

}

impl Display for GraphFrame {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let vertices = match self.vertices.clone().collect() { // TODO: explain why this is just fine
            Ok(vertices) => vertices,
            Err(error) => return std::fmt::Display::fmt(&error, f),
        };
        let edges = match self.edges.clone().collect() {
            Ok(edges) => edges,
            Err(error) => return std::fmt::Display::fmt(&error, f),
        };
        write!(f, "Vertices: {}\nEdges: {}", vertices, edges)
    }

}