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
    assert_true(@file.elements[2].instance_of? Section)

    text = @file.elements[2].children
    assert_equal(3, text.length)
    assert_equal(2, text[0].elements.length)
    assert_equal(5, text[1].elements.length)
  end

  def test_blocks
    text = @file.elements[3].children
    assert_true(text[0].is_a? Block)
    assert_true(text[0].instance_of? Comment)
  end

  def test_section_id
    assert_equal("first", @file.elements[1].id)
    assert_equal("and-another-top-level-section", @file.elements[2].id)
  end
end
