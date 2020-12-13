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
             :right_parenthesis]
  end

  def is_special_text_delimiter?
    is_any? [:asterisk, :slash]
  end
end

def tokens_to_s tokens
  s = ""
  tokens.each { |t| s << t.value }
  s
end
