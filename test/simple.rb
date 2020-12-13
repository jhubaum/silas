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

  def test_complex_text
    assert_true(@file.elements[-1].instance_of? Section)
    text = @file.elements[-1].children
    assert_equal(2, text.length)
  end
end
