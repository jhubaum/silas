require "./tokenizer"

class Token
  def is? kind
    @kind == kind
  end

  def is_any? kinds
    kinds.any? { |k| @kind == k }
  end

  def is_text?
    is_any? [:word, :whitespace, :minus, :hypen, :number, :dot, :comma,
             :question_mark, :exclamation_mark, :left_parenthesis,
             :right_parenthesis, :single_quote, :colon, :semicolon]
  end

  def is_special_text_delimiter?
    is_any? [:asterisk, :slash]
  end

  def is_text_element?
    is_text? or is_special_text_delimiter? or is?(:left_square_brace)
  end
end

def tokens_to_s tokens
  s = ""
  tokens.each { |t| s << t.value }
  s
end
