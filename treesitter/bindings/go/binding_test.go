package tree_sitter_hades_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_hades "github.com/tree-sitter/tree-sitter-hades/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_hades.Language())
	if language == nil {
		t.Errorf("Error loading Hades Parser grammar")
	}
}
