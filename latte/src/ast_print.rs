use std::fmt;

use ast::*;

pub trait Display {
    fn print0(&self) {
        let indent = String::from("");
        self.print(&indent);
    }

    fn print(&self, indent: &String);
}

impl Display for Program {
    fn print(&self, indent: &String) {
        let Program(ref defs) = *self;
        defs.print(indent);
    }
}

impl Display for Def {
    fn print(&self, indent: &String) {
        match *self {
            Def::DFunc(ref f, ref args, ref ret_type, ref stmt) => {
                println!("{}{} {}({})", indent, ret_type, f, print_vec(args));
                stmt.print(indent);
            }
        }
    }
}

impl Display for Stmt {
    fn print(&self, indent: &String) {
        match *self {
            Stmt::SBlock(ref stmts) => {
                println!("{}{}", indent, '{');
                {
                    let indent = indent.clone() + "\t";
                    stmts.print(&indent);
                }
                println!("{}{}", indent, '}');
            }
            Stmt::SDecl(ref t, ref inits) => {
                println!("{}{} {};", indent, t, print_vec(inits));
            }
            Stmt::SAssign(ref i, ref e) => {
                println!("{}{} = {};", indent, i, e);
            }
            Stmt::SInc(ref i) => println!("{}{}++;", indent, i),
            Stmt::SDec(ref i) => println!("{}{}--;", indent, i),
            Stmt::SReturnE(ref e) => println!("{}return {};", indent, e),
            Stmt::SReturn => println!("{}return;", indent),
            Stmt::SExpr(ref e) => println!("{}{};", indent, e),
            Stmt::SIf(ref cond, ref stmt) => {
                println!("{}if ({})", indent, cond);
                stmt.print(indent);
            }
            Stmt::SIfElse(ref cond, ref if_t, ref if_f) => {
                println!("{}if ({})", indent, cond);
                if_t.print(indent);
                println!("{}else", indent);
                if_f.print(indent);
            }
            Stmt::SWhile(ref cond, ref stmt) => {
                println!("{}while ({})", indent, cond);
                stmt.print(indent);
            }
        }
    }
}

impl<T> Display for Vec<T>
    where T: Display
{
    fn print(&self, indent: &String) {
        for x in self {
            x.print(indent);
        }
    }
}

impl fmt::Display for FuncArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let FuncArg(ref t, ref ident) = *self;
        write!(f, "{} {}", t, ident)
    }
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            VarDecl::Init(ref name, ref e) => format!("{} = {}", name, e),
            VarDecl::NoInit(ref name) => format!("{}", name),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Expr::EVar(ref i) => format!("{}", i),
            Expr::ELit(ref i) => format!("{}", i),
            Expr::ECall(ref f, ref args) => format!("{}({})", f, print_vec(args)),
            Expr::ENeg(ref e) => format!("-{}", *e),
            Expr::ENot(ref e) => format!("!{}", *e),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => format!("({} {} {})", *lhs, op, *rhs),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Lit::LInt(x) => format!("{}", x),
            Lit::LTrue => format!("true"),
            Lit::LFalse => format!("false"),
            Lit::LString(ref s) => format!("\"{}\"", s),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref s) = *self;
        write!(f, "{}", s)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Type::TInt => "int",
            Type::TString => "string",
            Type::TBool => "boolean",
            Type::TVoid => "void",
            _ => "?type?",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Operator::OpAdd => "+",
            Operator::OpSub => "-",
            Operator::OpMul => "*",
            Operator::OpDiv => "/",
            Operator::OpLess => "<",
            Operator::OpGreater => ">",
            Operator::OpLessE => "<=",
            Operator::OpGreaterE => ">=",
            Operator::OpEq => "==",
            Operator::OpNEq => "!=",
            Operator::OpAnd => "&&",
            Operator::OpOr => "||",
        };
        write!(f, "{}", s)
    }
}

fn print_vec<T>(vec: &Vec<T>) -> String
    where T: fmt::Display
{
    let mut s = String::new();
    for x in vec {
        if !s.is_empty() {
            s.push_str(", ");
        }
        s.push_str(format!("{}", x).as_str());
    }
    s
}
