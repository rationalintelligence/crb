use anyhow::Error;

#[derive(Default)]
pub struct Failures {
    errors: Vec<Error>,
}

impl Failures {
    pub fn put(&mut self, res: Result<(), Error>) {
        if let Err(err) = res {
            self.errors.push(err);
        }
    }
}
