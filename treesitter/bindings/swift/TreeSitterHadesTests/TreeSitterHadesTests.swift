import XCTest
import SwiftTreeSitter
import TreeSitterHades

final class TreeSitterHadesTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_hades())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Hades Parser grammar")
    }
}
