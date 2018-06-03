#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SeekPos {
    TopFirst,
    TopLast,
    BottomFirst,
    BottomLast,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NavigateDir {
    Up(bool),
    Down,
    Left,
    Right,
}

pub trait BiIterator: Iterator {
    fn prev(&mut self) -> Option<Self::Item>;
}

pub trait Navigator {
    type Item;

    fn navigate(&mut self, dir: NavigateDir) -> Option<Self::Item>;

    fn seek(&mut self, pos: SeekPos) -> Option<Self::Item>;

    fn down(&mut self) -> Option<Self::Item> {
        self.navigate(NavigateDir::Down)
    }

    fn right(&mut self) -> Option<Self::Item> {
        self.navigate(NavigateDir::Right)
    }

    fn left(&mut self) -> Option<Self::Item> {
        self.navigate(NavigateDir::Left)
    }

    fn up(&mut self) -> Option<Self::Item> {
        self.navigate(NavigateDir::Up(true))
    }

    fn exit(&mut self) -> Option<Self::Item> {
        self.navigate(NavigateDir::Up(false))
    }
}
