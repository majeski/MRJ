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
    SEmpty,
    SBlock(Vec<Stmt>),
    SDecl(Type, Vec<VarDecl>),
    SAssign(FieldGet, Expr),
    SInc(FieldGet),
    SDec(FieldGet),
    SReturnE(Expr),
    SReturn,
    SExpr(Expr),
    SIf(Expr, Box<Stmt>),
    SIfElse(Expr, Box<Stmt>, Box<Stmt>),
    SWhile(Expr, Box<Stmt>),
    SFor(Type, Ident, Expr, Box<Stmt>),
}

#[derive(Debug, Clone)]
pub enum VarDecl {
    Init(Type, Ident, Expr),
    NoInit(Type, Ident),
}

#[derive(Debug, Clone)]
pub enum Expr {
    EVar(FieldGet),
    ELit(Lit),
    ECall(FieldGet, Vec<Expr>),
    ENeg(Box<Expr>),
    ENot(Box<Expr>),
    EBinOp(Box<Expr>, Operator, Box<Expr>),
    ENew(Type),
    ENewArray(Type, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Lit {
    LInt(i32),
    LTrue,
    LFalse,
    LString(String),
    LNull(Option<Ident>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum FieldGet {
    Indirect(Box<Expr>, Ident), // <expr>.field
    Direct(Ident),
    IdxAccess(Box<Expr>, Box<Expr>), // <expr>[<expr>]
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
    TArray(Box<Type>),
    TObject(Ident /* class name */),
    TNull,
}
