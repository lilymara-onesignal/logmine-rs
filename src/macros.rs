macro_rules! vec_into {
    ($($x:expr),+ $(,)?) => ({
        let mut v = Vec::new();

        $(
            v.push($x.into());
        )*

        v.into_iter().collect()
    })
}
