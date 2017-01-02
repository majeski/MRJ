#[derive(Debug, Clone)]
pub struct Program(pub Vec<Def>);

#[derive(Debug, Clone)]
pub enum Def {
    DClass(Class),
    DFunc(Func),
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Ident,
    pub superclass: Option<Ident>,
    pub vars: Vec<Var>,
    pub methods: Vec<Func>,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub ident: Ident,
    pub args: Vec<Var>,
    pub ret_type: Type,
    pub body: Vec<Stmt>,
}

impl Func {
    pub fn get_type(&self) -> Type {
        Type::TFunc(self.args.iter().map(|v| v.t.clone()).collect(),
                    Box::new(self.ret_type.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinFunc {
    pub ident: Ident,
    pub args: Vec<Type>,
    pub ret_type: Type,
}

impl BuiltinFunc {
    pub fn get_type(&self) -> Type {
        Type::TFunc(self.args.clone(), Box::new(self.ret_type.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    pub t: Type,
    pub ident: Ident,
}

impl Var {
    pub fn get_type(&self) -> Type {
        self.t.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    SBlock(Vec<Stmt>),
    SDecl(Type, Vec<VarDecl>),
    SAssign(FieldGet, Expr),
    SInc(FieldGet),
    SDec(FieldGet),
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
    EVar(FieldGet),
    ELit(Lit),
    ECall(FieldGet, Vec<Expr>),
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
    LNull,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpMod,
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
pub struct FieldGet {
    pub ident: Ident,
    pub field: Option<Box<FieldGet>>,
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
    TObject(Ident /* class name */),
    TNull,
}
