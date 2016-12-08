#[derive(Debug, Clone)]
pub struct Program(pub Vec<Def>);

#[derive(Debug, Clone)]
pub enum Def {
    DFunc(Ident, Vec<FuncArg>, Type, Stmt),
}

#[derive(Debug, Clone)]
pub struct FuncArg(pub Type, pub Ident);

#[derive(Debug, Clone)]
pub enum Stmt {
    SBlock(Vec<Stmt>),
    SDecl(Type, Vec<VarDecl>),
    SAssign(Ident, Expr),
    SInc(Ident),
    SDec(Ident),
    SReturnE(Expr),
    SReturn,
    SExpr(Expr),
    SIf(Expr, Box<Stmt>),
    SIfElse(Expr, Box<Stmt>, Box<Stmt>),
    SWhile(Expr, Box<Stmt>),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    Init(Ident, Expr),
    NoInit(Ident),
}

#[derive(Debug, Clone)]
pub enum Expr {
    EVar(Ident),
    ELit(Lit),
    ECall(Ident, Vec<Expr>),
    ENeg(Box<Expr>),
    ENot(Box<Expr>),
    EBinOp(Box<Expr>, Operator, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Lit {
    LInt(i32),
    LTrue,
    LFalse,
    LString(String),
}

#[derive(Debug, Clone)]
pub enum Operator {
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpLess,
    OpGreater,
    OpLessE,
    OpGreaterE,
    OpEq,
    OpNEq,
    OpAnd,
    OpOr,
}

#[derive(Debug, Clone)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    TInt,
    TString,
    TBool,
    TVoid,
    TFunc(Vec<Type>, Box<Type>),
}
