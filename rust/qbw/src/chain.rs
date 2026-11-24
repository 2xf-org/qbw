//! The order of a transaction's lines.

use crate::ids::LineId;

/// A line that names the one after it.
pub(crate) trait Linked {
    fn id(&self) -> LineId;

    /// The next line, or `None` on the last.
    fn next(&self) -> Option<LineId>;
}

/// The lines of a transaction in the order of its form: from its own line on,
/// each names the one after it.
#[derive(Debug)]
pub(crate) struct Chain<T> {
    /// The lines that the chain reaches, and then the others by their ids.
    pub(crate) lines: Vec<T>,
    /// The number of lines that the chain reaches.
    pub(crate) reached: usize,
    /// Whether the chain reaches every line and ends with the last. Otherwise
    /// it is broken: a line of it is lost, or it goes round.
    pub(crate) is_whole: bool,
}

impl<T: Linked> Chain<T> {
    /// Orders the lines of a transaction by their chain from `own` on, which is
    /// its own line.
    pub(crate) fn new(mut lines: Vec<T>, own: LineId) -> Self {
        lines.sort_unstable_by_key(T::id);
        let mut places: Vec<Option<usize>> = vec![None; lines.len()];
        let mut reached = 0;
        let mut next = Some(own);
        while let Some((line, place)) = next.and_then(|id| at(&lines, &mut places, id))
            && place.is_none()
        {
            *place = Some(reached);
            reached += 1;
            next = line.next();
        }
        let mut placed: Vec<(Option<usize>, T)> = places.into_iter().zip(lines).collect();
        placed.sort_by_key(|(place, _)| place.unwrap_or(usize::MAX));
        Self {
            is_whole: next.is_none() && reached == placed.len(),
            lines: placed.into_iter().map(|(_, line)| line).collect(),
            reached,
        }
    }
}

/// The line `id` of `lines`, which are ordered by their ids, with its place.
fn at<'lines, T: Linked>(
    lines: &'lines [T],
    places: &'lines mut [Option<usize>],
    id: LineId,
) -> Option<(&'lines T, &'lines mut Option<usize>)> {
    let position = lines.binary_search_by_key(&id, T::id).ok()?;
    lines.get(position).zip(places.get_mut(position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Link {
        id: i32,
        next: i32,
    }

    impl Linked for Link {
        fn id(&self) -> LineId {
            LineId::new(self.id)
        }

        fn next(&self) -> Option<LineId> {
            (self.next != 0).then(|| LineId::new(self.next))
        }
    }

    /// The ids of lines in the order of their chain from the line 1 on, with a
    /// bar where it breaks. Each line is given with the next.
    fn ordered<const N: usize>(lines: [(i32, i32); N]) -> String {
        let links = lines.map(|(id, next)| Link { id, next });
        let chain = Chain::new(links.into(), LineId::new(1));
        let ids = chain.lines.iter().map(|link| link.id.to_string());
        let mut ids: Vec<String> = ids.collect();
        if !chain.is_whole {
            ids.insert(chain.reached, "|".to_owned());
        }
        ids.join(" ")
    }

    #[test]
    fn lines_follow_each_other() {
        assert_eq!(ordered([(3, 0), (1, 7), (7, 3)]), "1 7 3");
        assert_eq!(ordered([(1, 0)]), "1");
    }

    #[test]
    fn the_lines_past_a_break_are_ordered_by_their_ids() {
        assert_eq!(ordered([(9, 4), (1, 2), (4, 0), (2, 8)]), "1 2 | 4 9");
        assert_eq!(ordered([(1, 2)]), "1 |");
        assert_eq!(ordered([(1, 0), (2, 0)]), "1 | 2");
    }

    #[test]
    fn a_ring_and_a_chain_without_a_start_end() {
        assert_eq!(ordered([(1, 2), (2, 1)]), "1 2 |");
        assert_eq!(ordered([(2, 3), (3, 0)]), "| 2 3");
        assert_eq!(ordered([]), "|");
    }
}
