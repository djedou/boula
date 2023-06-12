mod entry;
mod key;
mod log;
mod scan;


pub use entry::*;
pub use key::*;
pub use self::log::*;
pub use scan::*;

/*

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn setup() -> Result<(Log, Box<log::Test>)> {
        let store = Box::new(log::Test::new());
        let log = Log::new(store.clone())?;
        Ok((log, store))
    }

    #[test]
    fn new() -> Result<()> {
        let (l, _) = setup()?;
        assert_eq!(0, l.last_index);
        assert_eq!(0, l.last_term);
        assert_eq!(0, l.commit_index);
        assert_eq!(0, l.commit_term);
        assert_eq!(None, l.get(1)?);
        Ok(())
    }

    #[test]
    fn append() -> Result<()> {
        let (mut l, _) = setup()?;
        assert_eq!(Ok(None), l.get(1));

        assert_eq!(
            Entry { index: 1, term: 3, command: Some(vec![0x01]) },
            l.append(3, Some(vec![0x01]))?
        );
        assert_eq!(Some(Entry { index: 1, term: 3, command: Some(vec![0x01]) }), l.get(1)?);
        assert_eq!(None, l.get(2)?);

        assert_eq!(1, l.last_index);
        assert_eq!(3, l.last_term);
        assert_eq!(0, l.commit_index);
        assert_eq!(0, l.commit_term);
        Ok(())
    }

    #[test]
    fn append_none() -> Result<()> {
        let (mut l, _) = setup()?;
        assert_eq!(Entry { index: 1, term: 3, command: None }, l.append(3, None)?);
        assert_eq!(Some(Entry { index: 1, term: 3, command: None }), l.get(1)?);
        Ok(())
    }

    #[test]
    fn append_persistence() -> Result<()> {
        let (mut l, store) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, None)?;
        l.append(2, Some(vec![0x03]))?;

        let l = Log::new(store)?;
        assert_eq!(Some(Entry { index: 1, term: 1, command: Some(vec![0x01]) }), l.get(1)?);
        assert_eq!(Some(Entry { index: 2, term: 2, command: None }), l.get(2)?);
        assert_eq!(Some(Entry { index: 3, term: 2, command: Some(vec![0x03]) }), l.get(3)?);
        Ok(())
    }

    #[test]
    fn commit() -> Result<()> {
        let (mut l, store) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, None)?;
        l.append(2, Some(vec![0x03]))?;
        assert_eq!(3, l.commit(3)?);
        assert_eq!(3, l.commit_index);
        assert_eq!(2, l.commit_term);

        // The last committed entry must be persisted, to sync with state machine
        let l = Log::new(store)?;
        assert_eq!(3, l.commit_index);
        assert_eq!(2, l.commit_term);
        Ok(())
    }

    #[test]
    fn commit_beyond() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, None)?;
        l.append(2, Some(vec![0x03]))?;
        assert_eq!(Err(Error::Internal("Entry 4 not found".into())), l.commit(4));

        Ok(())
    }

    #[test]
    fn commit_partial() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, None)?;
        l.append(2, Some(vec![0x03]))?;
        assert_eq!(2, l.commit(2)?);
        assert_eq!(2, l.commit_index);
        assert_eq!(2, l.commit_term);
        Ok(())
    }

    #[test]
    fn get() -> Result<()> {
        let (mut l, _) = setup()?;
        assert_eq!(None, l.get(1)?);

        l.append(3, Some(vec![0x01]))?;
        assert_eq!(Some(Entry { index: 1, term: 3, command: Some(vec![0x01]) }), l.get(1)?);
        assert_eq!(None, l.get(2)?);
        Ok(())
    }

    #[test]
    fn has() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(2, Some(vec![0x01]))?;

        assert_eq!(true, l.has(1, 2)?);
        assert_eq!(true, l.has(0, 0)?);
        assert_eq!(false, l.has(0, 1)?);
        assert_eq!(false, l.has(1, 0)?);
        assert_eq!(false, l.has(1, 3)?);
        assert_eq!(false, l.has(2, 0)?);
        assert_eq!(false, l.has(2, 1)?);
        Ok(())
    }

    #[test]
    fn scan() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(1, Some(vec![0x02]))?;
        l.append(1, Some(vec![0x03]))?;

        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 1, command: Some(vec![0x02]) },
                Entry { index: 3, term: 1, command: Some(vec![0x03]) },
            ],
            l.scan(0..).collect::<Result<Vec<_>>>()?
        );
        assert_eq!(
            vec![
                Entry { index: 2, term: 1, command: Some(vec![0x02]) },
                Entry { index: 3, term: 1, command: Some(vec![0x03]) },
            ],
            l.scan(2..).collect::<Result<Vec<_>>>()?
        );
        assert!(l.scan(4..).collect::<Result<Vec<_>>>()?.is_empty());
        Ok(())
    }

    #[test]
    fn load_save_term() -> Result<()> {
        // Test loading empty term
        let (l, _) = setup()?;
        assert_eq!((0, None), l.load_term()?);

        // Test loading saved term
        let (mut l, store) = setup()?;
        l.save_term(1, Some("a"))?;
        let l = Log::new(store)?;
        assert_eq!((1, Some("a".into())), l.load_term()?);

        // Test replacing saved term with none
        let (mut l, _) = setup()?;
        l.save_term(1, Some("a"))?;
        assert_eq!((1, Some("a".into())), l.load_term()?);
        l.save_term(0, None)?;
        assert_eq!((0, None), l.load_term()?);
        Ok(())
    }

    #[test]
    fn splice() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(
            4,
            l.splice(vec![
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
                Entry { index: 4, term: 4, command: Some(vec![0x04]) },
            ])?
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
                Entry { index: 4, term: 4, command: Some(vec![0x04]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        assert_eq!(4, l.last_index);
        assert_eq!(4, l.last_term);
        Ok(())
    }

    #[test]
    fn splice_all() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(
            2,
            l.splice(vec![
                Entry { index: 1, term: 4, command: Some(vec![0x0a]) },
                Entry { index: 2, term: 4, command: Some(vec![0x0b]) },
            ])?
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 4, command: Some(vec![0x0a]) },
                Entry { index: 2, term: 4, command: Some(vec![0x0b]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        assert_eq!(2, l.last_index);
        assert_eq!(4, l.last_term);
        Ok(())
    }

    #[test]
    fn splice_append() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;

        assert_eq!(
            4,
            l.splice(vec![
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
                Entry { index: 4, term: 4, command: Some(vec![0x04]) },
            ])?
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
                Entry { index: 4, term: 4, command: Some(vec![0x04]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        assert_eq!(4, l.last_index);
        assert_eq!(4, l.last_term);
        Ok(())
    }

    #[test]
    fn splice_conflict_term() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;
        l.append(4, Some(vec![0x04]))?;

        assert_eq!(
            3,
            l.splice(vec![
                Entry { index: 2, term: 3, command: Some(vec![0x0b]) },
                Entry { index: 3, term: 3, command: Some(vec![0x0c]) }
            ])?
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 3, command: Some(vec![0x0b]) },
                Entry { index: 3, term: 3, command: Some(vec![0x0c]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        assert_eq!(3, l.last_index);
        assert_eq!(3, l.last_term);
        Ok(())
    }

    #[test]
    fn splice_error_noncontiguous() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(
            Err(Error::Internal("Spliced entries must be contiguous".into())),
            l.splice(vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
            ])
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    #[test]
    fn splice_error_beyond_last() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(
            Err(Error::Internal("Spliced entries cannot begin past last index".into())),
            l.splice(vec![
                Entry { index: 5, term: 3, command: Some(vec![0x05]) },
                Entry { index: 6, term: 3, command: Some(vec![0x06]) },
            ])
        );
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    #[test]
    fn splice_overlap_inside() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(3, l.splice(vec![Entry { index: 2, term: 2, command: Some(vec![0x02]) },])?);
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    #[test]
    fn truncate() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(2, l.truncate(2)?);
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    #[test]
    fn truncate_beyond() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(3, l.truncate(4)?);
        assert_eq!(
            vec![
                Entry { index: 1, term: 1, command: Some(vec![0x01]) },
                Entry { index: 2, term: 2, command: Some(vec![0x02]) },
                Entry { index: 3, term: 3, command: Some(vec![0x03]) },
            ],
            l.scan(..).collect::<Result<Vec<_>>>()?
        );
        Ok(())
    }

    #[test]
    fn truncate_committed() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;
        l.commit(2)?;

        assert_eq!(
            l.truncate(1),
            Err(Error::Internal("Cannot truncate below committed index 2".into()))
        );
        assert_eq!(l.truncate(2)?, 2);
        Ok(())
    }

    #[test]
    fn truncate_zero() -> Result<()> {
        let (mut l, _) = setup()?;
        l.append(1, Some(vec![0x01]))?;
        l.append(2, Some(vec![0x02]))?;
        l.append(3, Some(vec![0x03]))?;

        assert_eq!(0, l.truncate(0)?);
        assert!(l.scan(..).collect::<Result<Vec<_>>>()?.is_empty());
        Ok(())
    }
}

*/