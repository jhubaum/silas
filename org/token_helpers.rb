require_relative "tokenizer"

class Token
  def is? kind
    @kind == kind
  end

  def is_any? kinds
    kinds.any? { |k| @kind == k }
  end

  def is_line_end?
    is_any? [:newline, :section_start]
  end

  def is_paragraph_end?
    is_any? [:newline, :section_start, :block_start, :attribute_start,
             :minus, :plus, :number]
  end

  def is_list_start?
    is_any? [:whitespace, :plus, :minus, :number]
  end
end

def tokens_to_s tokens
  s = ""
  tokens.each { |t| s << t.value }
  s
end
