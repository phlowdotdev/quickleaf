#[derive(Debug)]
pub enum Filter<'a> {
    StartWith(&'a str),
    EndWith(&'a str),
    StartAndEndWith(&'a str, &'a str),
    None,
}

impl<'a> Default for Filter<'a> {
    fn default() -> Self {
        Self::None
    }
}
