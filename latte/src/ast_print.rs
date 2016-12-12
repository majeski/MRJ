use std::fmt;

use ast::*;

static FERR: &'static str = "Unexpected format error";

pub fn print_code(p: &Program) {
    print!("{}", p);
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

// Display adds indentation to standard fmt::Display
trait Display {
    fn print0(&self, dst: &mut fmt::Write) {
        self.print(&String::new(), dst);
    }

    fn print(&self, indent: &String, dst: &mut fmt::Write);
}

impl Display for Program {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let Program(ref defs) = *self;
        defs.print(indent, dst);
    }
}

impl fmt::Display for Def {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

impl Display for Def {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let inner_indent = format!("{}\t", indent);
        match *self {
            Def::DFunc(ref f, ref args, ref ret_type, ref stmts) => {
                writeln!(dst,
                         "{}{} {}({}) {}",
                         indent,
                         ret_type,
                         f,
                         print_vec(args),
                         '{')
                    .expect(FERR);
                stmts.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print0(f);
        Ok(())
    }
}

impl Display for Stmt {
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        let inner_indent = format!("{}\t", indent);
        match *self {
            Stmt::SBlock(ref stmts) => {
                writeln!(dst, "{}{}", indent, '{').expect(FERR);
                stmts.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SDecl(ref t, ref inits) => {
                writeln!(dst, "{}{} {};", indent, t, print_vec(inits)).expect(FERR)
            }

            Stmt::SAssign(ref i, ref e) => writeln!(dst, "{}{} = {};", indent, i, e).expect(FERR),

            Stmt::SInc(ref i) => writeln!(dst, "{}{}++;", indent, i).expect(FERR),
            Stmt::SDec(ref i) => writeln!(dst, "{}{}--;", indent, i).expect(FERR),
            Stmt::SReturnE(ref e) => writeln!(dst, "{}return {};", indent, e).expect(FERR),
            Stmt::SReturn => writeln!(dst, "{}return;", indent).expect(FERR),
            Stmt::SExpr(ref e) => writeln!(dst, "{}{};", indent, e).expect(FERR),
            Stmt::SIf(ref cond, ref stmts) => {
                writeln!(dst, "{}if ({}) {}", indent, cond, '{').expect(FERR);
                let inner_indent = format!("{}\t", indent);
                stmts.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SIfElse(ref cond, ref if_t, ref if_f) => {
                writeln!(dst, "{}if ({}) {}", indent, cond, '{').expect(FERR);
                if_t.print(&inner_indent, dst);
                writeln!(dst, "{}{} else {}", indent, '}', '{').expect(FERR);
                if_f.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
            Stmt::SWhile(ref cond, ref stmts) => {
                writeln!(dst, "{}while ({}) {}", indent, cond, '{').expect(FERR);
                stmts.print(&inner_indent, dst);
                writeln!(dst, "{}{}", indent, '}').expect(FERR);
            }
        };
    }
}

impl<T> Display for Vec<T>
    where T: Display
{
    fn print(&self, indent: &String, dst: &mut fmt::Write) {
        for x in self {
            x.print(indent, dst);
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
            Type::TFunc(_, _) => "<func_type>",
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
