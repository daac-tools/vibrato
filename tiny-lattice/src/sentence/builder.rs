use super::Sentence;

impl Sentence {
    /// Creates new InputBuffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the input buffer, so it could be used to process new input.
    /// New input should be written to the returned mutable reference.
    pub fn reset(&mut self) -> &mut String {
        // extended buffers can be ignored during cleaning,
        // they will be cleaned before usage automatically
        self.original.clear();
        self.modified.clear();
        self.m2o.clear();
        self.mod_chars.clear();
        self.mod_c2b.clear();
        self.mod_b2c.clear();
        self.mod_bow.clear();
        &mut self.original
    }

    // /// Moves InputBuffer into RW state, making it possible to perform edits on it
    // pub fn start_build(&mut self) -> SudachiResult<()> {
    //     if self.original.len() > MAX_LENGTH {
    //         return Err(SudachiError::InputTooLong(self.original.len(), MAX_LENGTH));
    //     }
    //     debug_assert_eq!(self.state, BufferState::Clean);
    //     self.state = BufferState::RW;
    //     self.modified.push_str(&self.original);
    //     self.m2o.extend(0..self.modified.len() + 1);
    //     Ok(())
    // }

    // /// Finalizes InputBuffer state, making it RO
    // pub fn build(&mut self, grammar: &Grammar) -> SudachiResult<()> {
    //     debug_assert_eq!(self.state, BufferState::RW);
    //     self.state = BufferState::RO;
    //     self.mod_chars.clear();
    //     let cats = &grammar.character_category;
    //     let mut last_offset = 0;
    //     let mut last_chidx = 0;

    //     // Special cases for BOW logic
    //     let non_starting = CategoryType::ALPHA | CategoryType::GREEK | CategoryType::CYRILLIC;
    //     let mut prev_cat = CategoryType::empty();
    //     self.mod_bow.resize(self.modified.len(), false);
    //     let mut next_bow = true;

    //     for (chidx, (bidx, ch)) in self.modified.char_indices().enumerate() {
    //         self.mod_chars.push(ch);
    //         let cat = cats.get_category_types(ch);
    //         self.mod_cat.push(cat);
    //         self.mod_c2b.push(bidx);
    //         self.mod_b2c
    //             .extend(std::iter::repeat(last_chidx).take(bidx - last_offset));
    //         last_offset = bidx;
    //         last_chidx = chidx;

    //         let can_bow = if !next_bow {
    //             // this char was forbidden by the previous one
    //             next_bow = true;
    //             false
    //         } else if cat.intersects(CategoryType::NOOOVBOW2) {
    //             // this rule is stronger than the next one and must come before
    //             // this and next are forbidden
    //             next_bow = false;
    //             false
    //         } else if cat.intersects(CategoryType::NOOOVBOW) {
    //             // this char is forbidden
    //             false
    //         } else if cat.intersects(non_starting) {
    //             // the previous char is compatible
    //             !cat.intersects(prev_cat)
    //         } else {
    //             true
    //         };

    //         self.mod_bow[bidx] = can_bow;
    //         prev_cat = cat;
    //     }
    //     // trailing indices for the last codepoint
    //     self.mod_b2c
    //         .extend(std::iter::repeat(last_chidx).take(self.modified.len() - last_offset));
    //     // sentinel values for range translations
    //     self.mod_c2b.push(self.mod_b2c.len());
    //     self.mod_b2c.push(last_chidx + 1);

    //     self.fill_cat_continuity();
    //     self.fill_orig_b2c();

    //     Ok(())
    // }

    // fn fill_cat_continuity(&mut self) {
    //     if self.mod_chars.is_empty() {
    //         return;
    //     }
    //     // single pass algorithm
    //     // by default continuity is 1 codepoint
    //     // go from the back and set it prev + 1 when chars are compatible
    //     self.mod_cat_continuity.resize(self.mod_chars.len(), 1);
    //     let mut cat = *self.mod_cat.last().unwrap_or(&CategoryType::all());
    //     for i in (0..self.mod_cat.len() - 1).rev() {
    //         let cur = self.mod_cat[i];
    //         let common = cur & cat;
    //         if !common.is_empty() {
    //             self.mod_cat_continuity[i] = self.mod_cat_continuity[i + 1] + 1;
    //             cat = common;
    //         } else {
    //             cat = cur;
    //         }
    //     }
    // }

    // fn fill_orig_b2c(&mut self) {
    //     self.m2o_2.clear();
    //     self.m2o_2.resize(self.original.len() + 1, usize::MAX);
    //     let mut max = 0;
    //     for (ch_idx, (b_idx, _)) in self.original.char_indices().enumerate() {
    //         self.m2o_2[b_idx] = ch_idx;
    //         max = ch_idx
    //     }
    //     self.m2o_2[self.original.len()] = max + 1;
    // }

    // fn commit(&mut self) -> SudachiResult<()> {
    //     if self.replaces.is_empty() {
    //         return Ok(());
    //     }

    //     self.mod_chars.clear();
    //     self.modified_2.clear();
    //     self.m2o_2.clear();

    //     let sz = edit::resolve_edits(
    //         &self.modified,
    //         &self.m2o,
    //         &mut self.modified_2,
    //         &mut self.m2o_2,
    //         &mut self.replaces,
    //     );
    //     if sz > REALLY_MAX_LENGTH {
    //         // super improbable, but still
    //         return Err(SudachiError::InputTooLong(sz, REALLY_MAX_LENGTH));
    //     }
    //     std::mem::swap(&mut self.modified, &mut self.modified_2);
    //     std::mem::swap(&mut self.m2o, &mut self.m2o_2);
    //     Ok(())
    // }

    // fn rollback(&mut self) {
    //     self.replaces.clear()
    // }

    // fn make_editor<'a>(&mut self) -> InputEditor<'a> {
    //     // SAFETY: while it is possible to write into borrowed replaces
    //     // the buffer object itself will be accessible as RO
    //     let replaces: &'a mut Vec<edit::ReplaceOp<'a>> =
    //         unsafe { std::mem::transmute(&mut self.replaces) };
    //     return InputEditor::new(replaces);
    // }

    // /// Execute a function which can modify the contents of the current buffer
    // ///
    // /// Edit can borrow &str from the context with the borrow checker working correctly
    // pub fn with_editor<'a, F>(&mut self, func: F) -> SudachiResult<()>
    // where
    //     F: FnOnce(&InputBuffer, InputEditor<'a>) -> SudachiResult<InputEditor<'a>>,
    //     F: 'a,
    // {
    //     debug_assert_eq!(self.state, BufferState::RW);
    //     // InputBufferReplacer should have 'a lifetime parameter for API safety
    //     // It is impossible to create it outside of this function
    //     // And the API forces user to return it by value
    //     let editor: InputEditor<'a> = self.make_editor();
    //     match func(self, editor) {
    //         Ok(_) => self.commit(),
    //         Err(e) => {
    //             self.rollback();
    //             Err(e)
    //         }
    //     }
    // }

    // /// Recompute chars from modified string (useful if the processing will use chars)
    // pub fn refresh_chars(&mut self) {
    //     debug_assert_eq!(self.state, BufferState::RW);
    //     if self.mod_chars.is_empty() {
    //         self.mod_chars.extend(self.modified.chars());
    //     }
    // }
}
