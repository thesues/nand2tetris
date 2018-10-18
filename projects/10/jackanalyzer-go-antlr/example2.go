// example2.go
package main

import (
	"fmt"

	"./parser"
	"github.com/antlr/antlr4/runtime/Go/antlr"
)

type actionListener struct {
	parser.BaseJackListener
}

func getTokenType(t antlr.Token) string {
	switch t.GetTokenType() {
	case parser.JackParserINTEGER:
		return "integerConstant"
	case parser.JackParserSTRING:
		return "stringConstant"
	case parser.JackParserID:
		return "identifier"
	}
	//keyword and symbols are not defined in g4 file,
	if len(t.GetText()) == 1 {
		return "symbol"
	} else {
		return "keyword"
	}

}

/*
func (s *actionListener) VisitTerminal(node antlr.TerminalNode) {
	token := node.GetSymbol()
	tokenType := getTokenType(token)
	fmt.Printf("<%s>%s</%s>\n", tokenType, token.GetText(), tokenType)
}
*/

func (s *actionListener) EnterClazz(c *parser.ClazzContext) {
	fmt.Println("<class>")
}

func (s *actionListener) ExitClazz(c *parser.ClazzContext) {
	fmt.Println("</class>")
}

func (s *actionListener) EnterParameterList(c *parser.ParameterListContext) {
	fmt.Println("<parameterList>")
}

func (s *actionListener) ExitParameterList(c *parser.ParameterListContext) {
	fmt.Println("</parameterList>")
}

/*
func (s *actionListener) EnterTypedVar(c *parser.TypedVarContext) {
	fmt.Println("<identifier>")
	fmt.Println(c.VarName().GetText())
	fmt.Println("</identifier>")
}

func (s *actionListener) ExitTypedVar(c *parser.TypedVarContext) {
	fmt.Println("<symbol>, </symbol>")
}
*/

func (s *actionListener) EnterSubroutineBody(c *parser.SubroutineBodyContext) {
	fmt.Println("<subroutineBody>")
}

func (s *actionListener) ExitSubroutineBody(c *parser.SubroutineBodyContext) {
	fmt.Println("</subroutineBody>")
}

func (s *actionListener) EnterVarDec(c *parser.VarDecContext) {
	fmt.Println("<varDec>")
	/*
		fmt.Println("<keyword> var </keyword>")
		fmt.Println("<identifier>" + c.DataType().GetText() + "</identifier>")
	*/

}
func (s *actionListener) ExitVarDec(c *parser.VarDecContext) {
	//fmt.Println("<symbol> ; </symbol>")
	fmt.Println("</varDec>")
}

func (s *actionListener) EnterVarList(c *parser.VarListContext) {
	/*
		numberOfVars := len(c.AllVarName())
		for n, i := range c.AllVarName() {
			fmt.Println("<identifier>" + i.GetText() + "</identifier>")
			if n < numberOfVars-1 {
				fmt.Println("<symbol> , <symbol>")
			}
		}
	*/
}

func (s *actionListener) EnterSubroutineDec(c *parser.SubroutineDecContext) {
	fmt.Println("<subroutineDec>")
	/*
		fmt.Println("<keyword>" + c.SubroutineType().GetText() + "</keyword>")
		fmt.Println("<keyword>" + c.ReturnType().GetText() + "</keyword>")
		fmt.Println("<identifier>" + c.SubroutineName().GetText() + "</identifier>")
	*/
}

func (s *actionListener) ExitSubroutineDec(c *parser.SubroutineDecContext) {
	fmt.Println("</subroutineDec>")
}

func (s *actionListener) EnterLetStatement(c *parser.LetStatementContext) {
	fmt.Println("<letStatement>")
	/*
		fmt.Println("<keyword> let </keyword>")
		fmt.Println("<identifier>" + c.VarName().GetText() + "</identifier>")
		fmt.Println("<symbol> = </symbol>")
	*/
}

func (s *actionListener) ExitLetStatement(c *parser.LetStatementContext) {
	fmt.Println("</letStatement>")
}

func (s *actionListener) ExitBinaryExpr(c *parser.BinaryExprContext) {
	fmt.Println(c.GetBop().GetText())
}

func (s *actionListener) ExitIntergerTerm(c *parser.IntergerTermContext) {
	fmt.Println(c.INTEGER().GetText())
}

func (s *actionListener) ExitStringTerm(c *parser.StringTermContext) {
	fmt.Println(c.STRING().GetText())
}

func (s *actionListener) ExitUnaryExpr(c *parser.UnaryExprContext) {
	fmt.Println(c.UnaryOp().GetText())
}

/*
func (s *actionListener) EnterTerm(c *parser.TermContext) {
}

func (s *actionListener) ExitTerm(c *parser.TermContext) {
	fmt.Println(c.INTEGER().GetText())
}
*/

func main() {
	// Setup the input
	is := antlr.NewInputStream("1 * 2 + 3 / 4 + (1-4)")
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

	// Finally parse the expression
	listener := actionListener{}
	//antlr.ParseTreeWalkerDefault.Walk(&listener, p.Clazz())
	antlr.ParseTreeWalkerDefault.Walk(&listener, p.Expression())
}
