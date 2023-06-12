mod hybrid;
mod memory;
mod mutex_reader;
mod range;
mod scan;
mod store;

pub use hybrid::*;
pub use memory::*;
pub use mutex_reader::*;
pub use range::*;
pub use scan::*;
pub use store::*;


/*
#[cfg(test)]
use crate::error::Error;

#[cfg(test)]
trait TestSuite<S: Store> {
    fn setup() -> Result<S>;

    fn test() -> Result<()> {
        Self::test_append()?;
        Self::test_commit_truncate()?;
        Self::test_get()?;
        Self::test_metadata()?;
        Self::test_scan()?;
        Ok(())
    }

    fn test_append() -> Result<()> {
        let mut s = Self::setup()?;
        assert_eq!(0, s.len());
        assert_eq!(1, s.append(vec![0x01])?);
        assert_eq!(2, s.append(vec![0x02])?);
        assert_eq!(3, s.append(vec![0x03])?);
        assert_eq!(3, s.len());
        assert_eq!(
            vec![vec![1], vec![2], vec![3]],
            s.scan(Range::from(..)).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    fn test_commit_truncate() -> Result<()> {
        let mut s = Self::setup()?;

        assert_eq!(0, s.committed());

        // Truncating an empty store should be fine.
        assert_eq!(0, s.truncate(0)?);

        s.append(vec![0x01])?;
        s.append(vec![0x02])?;
        s.append(vec![0x03])?;
        s.commit(1)?;
        assert_eq!(1, s.committed());

        // Truncating beyond the end should be fine.
        assert_eq!(3, s.truncate(4)?);
        assert_eq!(
            vec![vec![1], vec![2], vec![3]],
            s.scan(Range::from(..)).collect::<Result<Vec<_>>>()?
        );

        // Truncating a committed entry should error.
        assert_eq!(
            Err(Error::Internal("Cannot truncate below committed index 1".into())),
            s.truncate(0)
        );

        // Truncating above should work.
        assert_eq!(1, s.truncate(1)?);
        assert_eq!(vec![vec![1]], s.scan(Range::from(..)).collect::<Result<Vec<_>>>()?);

        Ok(())
    }

    fn test_get() -> Result<()> {
        let mut s = Self::setup()?;
        s.append(vec![0x01])?;
        s.append(vec![0x02])?;
        s.append(vec![0x03])?;
        assert_eq!(None, s.get(0)?);
        assert_eq!(Some(vec![0x01]), s.get(1)?);
        assert_eq!(None, s.get(4)?);
        Ok(())
    }

    fn test_metadata() -> Result<()> {
        let mut s = Self::setup()?;
        s.set_metadata(b"a", vec![0x01])?;
        assert_eq!(Some(vec![0x01]), s.get_metadata(b"a")?);
        assert_eq!(None, s.get_metadata(b"b")?);
        Ok(())
    }

    #[allow(clippy::reversed_empty_ranges)]
    fn test_scan() -> Result<()> {
        let mut s = Self::setup()?;
        s.append(vec![0x01])?;
        s.append(vec![0x02])?;
        s.append(vec![0x03])?;
        s.commit(2)?;

        assert_eq!(
            vec![vec![1], vec![2], vec![3]],
            s.scan(Range::from(..)).collect::<Result<Vec<_>>>()?
        );

        assert_eq!(vec![vec![1]], s.scan(Range::from(0..2)).collect::<Result<Vec<_>>>()?);
        assert_eq!(vec![vec![1], vec![2]], s.scan(Range::from(1..3)).collect::<Result<Vec<_>>>()?);
        assert_eq!(
            vec![vec![1], vec![2], vec![3]],
            s.scan(Range::from(1..=3)).collect::<Result<Vec<_>>>()?
        );
        assert!(s.scan(Range::from(3..1)).collect::<Result<Vec<_>>>()?.is_empty());
        assert!(s.scan(Range::from(1..1)).collect::<Result<Vec<_>>>()?.is_empty());
        assert_eq!(vec![vec![2]], s.scan(Range::from(2..=2)).collect::<Result<Vec<_>>>()?);
        assert_eq!(vec![vec![2], vec![3]], s.scan(Range::from(2..5)).collect::<Result<Vec<_>>>()?);

        assert!(s.scan(Range::from(..0)).collect::<Result<Vec<_>>>()?.is_empty());
        assert_eq!(vec![vec![1]], s.scan(Range::from(..=1)).collect::<Result<Vec<_>>>()?);
        assert_eq!(vec![vec![1], vec![2]], s.scan(Range::from(..3)).collect::<Result<Vec<_>>>()?);

        assert!(s.scan(Range::from(4..)).collect::<Result<Vec<_>>>()?.is_empty());
        assert_eq!(vec![vec![3]], s.scan(Range::from(3..)).collect::<Result<Vec<_>>>()?);
        assert_eq!(vec![vec![2], vec![3]], s.scan(Range::from(2..)).collect::<Result<Vec<_>>>()?);

        Ok(())
    }
}
*/