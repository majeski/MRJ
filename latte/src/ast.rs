#[derive(Debug, Clone)]
pub struct Program(pub Vec<Def>);

#[derive(Debug, Clone)]
pub enum Def {
    DFunc(Ident, Vec<FuncArg>, Type, Vec<Stmt>),
}

impl Def {
    pub fn get_ident(&self) -> &Ident {
        match *self {
            Def::DFunc(ref fname, _, _, _) => fname,
        }
    }

    pub fn get_type(&self) -> Type {
        match *self {
            Def::DFunc(_, ref args, ref ret_type, _) => {
                Type::TFunc(args.iter().map(|a| a.0.clone()).collect(),
                            Box::new(ret_type.clone()))
            }
        }
    }
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
    SIf(Expr, Vec<Stmt>),
    SIfElse(Expr, Vec<Stmt>, Vec<Stmt>),
    SWhile(Expr, Vec<Stmt>),
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    TInt,
    TString,
    TBool,
    TVoid,
    TFunc(Vec<Type>, Box<Type>),
}
