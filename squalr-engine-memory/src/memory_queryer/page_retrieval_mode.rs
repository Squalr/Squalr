bitflags::bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct PageRetrievalMode: u32 {
        const FROM_SETTINGS         = 1 << 0;
        const FROM_USER_MODE_MEMORY = 1 << 1;
        const FROM_NON_MODULES      = 1 << 2;
        const FROM_MODULES          = 1 << 3;
    }
}
