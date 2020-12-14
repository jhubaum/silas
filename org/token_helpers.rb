require_relative "tokenizer"

class Token
  def is? kind
    @kind == kind
  end

  def is_any? kinds
    kinds.any? { |k| @kind == k }
  end

  def is_text?
    # :left_square_brace isn't text because this conflicts with parsing links right now
    is_any? [:word, :whitespace, :minus, :hypen, :number, :dot, :comma,
             :question_mark, :exclamation_mark, :left_parenthesis,
             :right_parenthesis, :single_quote, :colon, :semicolon,
             :quotation_mark, :right_square_brace]
  end

  def is_special_text_delimiter?
    is_any? [:asterisk, :slash]
  end

  def is_text_element?
    is_text? or is_special_text_delimiter? or is?(:left_square_brace)
  end

  def is_paragraph_end?
    # remove this function again once the merging of parsed elements is reworked
    is_any? [:newline, :section_start, :block_start, :attribute_start]
  end
end

def tokens_to_s tokens
  s = ""
  tokens.each { |t| s << t.value }
  s
end
