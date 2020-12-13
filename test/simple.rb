require "./orgparse.rb"
require "test/unit"

class TestParsingSimple < Test::Unit::TestCase
  def setup
    @file = OrgFile.new "test/simple.org"
  end

  def test_preamble
    assert_equal("This is a simple org file for testing the parser",
                 @file.preamble[:title])
  end

  def test_paragraph_count
    assert_true(@file.elements[-1].instance_of? Section)

    # two paragraphs. the first has two lines, the second has one with some special elements
    text = @file.elements[-1].children
    assert_equal(2, text.length)
    assert_equal(2, text[0].elements.length)
    assert_equal(5, text[1].elements.length)
  end
end
