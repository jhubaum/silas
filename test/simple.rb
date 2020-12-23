require_relative "../org/parser.rb"
require "test/unit"

class TestParsingSimple < Test::Unit::TestCase
  def setup
    @file = OrgFile.new "test/simple.org", nil
  end

  def test_preamble
    assert_equal("This is a simple org file for testing the parser",
                 @file.preamble[:title])
  end

  def test_paragraph_count
    assert_true(@file.elements[2].instance_of? Section)

    text = @file.elements[2].elements
    assert_equal(6, text.length)
    assert_equal(2, text[0].elements.length)
    assert_equal(5, text[1].elements.length)
  end

  def test_blocks
    text = @file.elements[3].elements
    assert_true(text[0].is_a? Block)
    assert_true(text[0].instance_of? Comment)

    quote = text[2]
    assert_equal("I said something super smart", quote.elements[0])
    assert_equal("Someone quoteable", quote.quotee[0])
  end

  def test_section_id
    assert_equal("first", @file.elements[1].id)
    assert_equal("and-another-top-level-section", @file.elements[2].id)
  end
end
