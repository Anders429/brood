use crate::resource::Null;

pub trait Length {
    const LEN: usize;
}

impl Length for Null {
    const LEN: usize = 0;
}

impl<Resource, Resources> Length for (Resource, Resources)
where
    Resources: Length,
{
    const LEN: usize = Resources::LEN + 1;
}

#[cfg(test)]
mod tests {
    use super::Length;
    use crate::Resources;

    #[test]
    fn empty_length() {
        assert_eq!(<Resources!()>::LEN, 0);
    }

    #[test]
    fn single_length() {
        assert_eq!(<Resources!(u8)>::LEN, 1);
    }

    #[test]
    fn multiple_length() {
        assert_eq!(<Resources!(u8, u16, u32, u64)>::LEN, 4);
    }
}
