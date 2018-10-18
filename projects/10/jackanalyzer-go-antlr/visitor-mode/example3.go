// example3.go
//for vistor pattern
package main

import (
	"fmt"

	"./parser"
	"github.com/antlr/antlr4/runtime/Go/antlr"
)

type Visitor struct {
	*parser.BaseJackVisitor
}

func NewVisitor() *Visitor {
	return &Visitor{&parser.BaseJackVisitor{}}
}

func (v *Visitor) VisitChildren(node antlr.RuleNode) interface{} {
	for _, child := range node.GetChildren() {
		switch child := child.(type) {
		case antlr.TerminalNode:
			v.VisitTerminal(child)
		case antlr.ErrorNode:
			v.VisitErrorNode(child)
		case antlr.RuleNode:
			child.Accept(v)
		default:
			// can this happen??
		}
	}
	return nil
}

func (v *Visitor) VisitExpression(ctx *parser.ExpressionContext) interface{} {
	term := ctx.Term()
	if term == nil { //bop
		//fmt.Println(ctx.Expression(0).GetText())
		v.VisitExpression(ctx.Expression(0).(*parser.ExpressionContext))
		//fmt.Println(ctx.Expression(1).GetText())
		v.VisitExpression(ctx.Expression(1).(*parser.ExpressionContext))
		fmt.Println(ctx.GetBop().GetText())
	} else {
		v.VisitTerm(term.(*parser.TermContext))
	}
	return nil
}

func (v *Visitor) VisitTerm(ctx *parser.TermContext) interface{} {
	return v.VisitChildren(ctx)
	/*
		child := ctx.GetChild(0)
		switch child.(type) {
		case *parser.NumberContext:
			fmt.Println(child.(*parser.NumberContext).GetText())
		}
		return nil
	*/
}

//func (v *Visitor) VisitTerm()
func (v *Visitor) VisitNumber(ctx *parser.NumberContext) interface{} {
	fmt.Println(ctx.INTEGER().GetText())
	return nil
}

func (v *Visitor) VisitLetStatement(ctx *parser.LetStatementContext) interface{} {
	var varName parser.IVarNameContext = ctx.VarName()
	fmt.Println("varname:" + varName.GetText())
	v.VisitExpression(ctx.Expression().(*parser.ExpressionContext))
	return nil
}

func (v *Visitor) VisitStatements(ctx *parser.StatementsContext) interface{} {
	for _, s := range ctx.AllStatement() {
		v.VisitStatement(s.(*parser.StatementContext))
	}
	return nil
}

func (v *Visitor) VisitStatement(ctx *parser.StatementContext) interface{} {

	/*
		children := ctx.GetChildren()
		for _, child := range children {
			switch child.(type) {
			case *parser.LetStatementContext:
				v.VisitLetStatement(child.(*parser.LetStatementContext))
			}
		}
		return nil
	*/
	return v.VisitChildren(ctx)
}

func main() {
	// Setup the input
	is := antlr.NewInputStream("let xxx = 1 * 2 * 3 * 4;")
	/*
		is, err := antlr.NewFileStream("/Users/zhangdongmao/upstream/nand2tetris/projects/10/ArrayTest/Main.jack")
		if err != nil {
				return
		}
	*/

	// Create the Lexer
	lexer := parser.NewJackLexer(is)
	tokens := antlr.NewCommonTokenStream(lexer, antlr.TokenDefaultChannel)

	// Create the Parser
	p := parser.NewJackParser(tokens)

	v := NewVisitor()
	p.Statements().Accept(v)
}
