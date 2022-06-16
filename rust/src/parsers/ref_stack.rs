
/// A safe interface for keeping a stack of references to nested structs
pub struct RefStack<'a, T> {
    //I tried using a smallvec here. Somehow, it didn't help :<
    //Guess we shouldn't expect to see performance improvements until you try linearizing/arenazing/using a bump allocator.
    // stack: SmallVec<[*mut T; 10]>,
    stack: Vec<*mut T>,
    pub top: &'a mut T,
}

impl<'a, T> RefStack<'a, T> {
    pub fn push(&mut self, selection: impl FnOnce(&'a mut T)-> &'a mut T) {
        self.stack.push(self.top);
        //I'm not really sure how to argue this is sound other than "why the hell wouldn't it be". We know that we have exclusive access to self, which contains a &'a mut ref, so we should be allowed to get the &'a mut ref.
        self.top = selection(unsafe{ &mut *(self.top as *mut _) });
    }
    pub fn pop(&mut self) {
        //popping to the child to expose the parent is safe, because it can only be done by relinquishing the child, which was the sole borrower of the parent element. Relinquishment occurs in the overwriding of `top`.
        self.stack.pop();
        self.top = unsafe {
            &mut **self
                .stack
                .last()
                .unwrap_or_else(|| panic!("{} was popped to zero", std::any::type_name::<Self>()))
        };
    }
    pub fn new(r: &'a mut T) -> Self {
        let mut stack = Vec::new();
        stack.push(r as *mut T);
        Self {
            stack,
            top: r,
        }
    }
    pub fn len(&self)-> usize { self.stack.len() }
}
