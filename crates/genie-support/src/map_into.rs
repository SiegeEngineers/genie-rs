/// Helper trait to map a container of T to a container of some type F that implements From<T>.
/// For example, Result<T> to Result<From<T>>, or Option<T> to Option<From<T>>.
pub trait MapInto<T> {
    fn map_into(self) -> T;
}

impl<Source, Target, Error> MapInto<Result<Target, Error>> for Result<Source, Error>
where
    Target: From<Source>,
{
    fn map_into(self) -> Result<Target, Error> {
        self.map(|v| v.into())
    }
}

impl<Source, Target> MapInto<Option<Target>> for Option<Source>
where
    Target: From<Source>,
{
    fn map_into(self) -> Option<Target> {
        self.map(|v| v.into())
    }
}
