use crate::domain::Result;

/// daedalus 状态机流转的统一接口。
///
/// `pre_check` 必须保持只读；所有写入 `state.toml`、渲染 `state.md`
/// 或移动 workspace 的副作用只能发生在 `commit` 中。
pub trait StateTransition: Sized {
    /// 流转完成后的输出类型。
    type Output;

    /// 执行只读前置校验。
    fn pre_check(&self) -> Result<()>;

    /// 提交状态流转。
    fn commit(self) -> Result<Self::Output>;

    /// 应用状态流转，统一保证提交前先完成前置校验。
    fn apply(self) -> Result<Self::Output> {
        self.pre_check()?;
        self.commit()
    }
}
