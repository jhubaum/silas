module Tokenizer
  private
  # to find the proper symbol name: https://symbolnames.org/
  TOKEN_INFOS = [
    [/#\+/, :attribute_start],
    [/ /, :whitespace],
    [/[[:alpha:]]+/, :word],
    [/\d+/, :number],
    [/:/, :colon],
    [/;/, :semicolon],
    [/\n\*/, :section_start],
    [/\*/, :asterisk],
    [/\n/, :newline],
    [/<\d{4}-\d{2}-\d{2}>/, :date],
    [/-/, :minus],
    [/–/, :hypen],
    [/</, :less_than],
    [/>/, :greater_than],
    [/,/, :comma],
    [/\./, :dot],
    [/_/, :underscore],
    [/\[/, :left_square_brace],
    [/\]/, :right_square_brace],
    [/{/, :left_curly_brace],
    [/}/, :right_curly_brace],
    [/\(/, :left_parenthesis],
    [/\)/, :right_parenthesis],
    [/"/, :quotation_mark],
    [/'/, :single_quote],
    [/\//, :slash],
    [/\?/, :question_mark]
  ]
end

class TokenError < ::StandardError
end

class UnknownCharError < TokenError
end

class TokenListError < TokenError
end

class Token
  attr_reader :kind, :value

  def initialize kind, value
    @kind, @value = kind, value
  end

  def is? kind
    @kind == kind
  end

  def is_any? kinds
    kinds.any? { |k| @kind == k }
  end

  def to_s
    "<Token #{@kind}>"
  end
end

def tokens_to_s tokens
  s = ""
  tokens.each { |t| s << t.value }
  s
end

class TokenList
  def initialize tokens
    @tokens = tokens
  end

  def has_tokens?
    @tokens.length > 0
  end

  def peek
    @tokens.first
  end

  def pop
    result = @tokens[0]
    @tokens = @tokens[1..-1]
    result
  end

  def pop_if &block
    raise ArgumentError, "No block given in pop_if" unless block_given?
    pop.value if has_tokens? and block.call peek
  end

  def pop_expected kind
    result = pop
    raise TokenListError, "pop_expected found type #{result.kind} but expected #{kind}" unless result.is? kind

    result.value
  end

  def pop_while &block
    raise ArgumentError, "No block given in pop_while" unless block_given?
    result = []
    while has_tokens? and block.call peek
      result << pop
    end
    result
  end

  def pop_until &block
    raise ArgumentError, "No block given in pop_until" unless block_given?
    pop_while { |t| not block.call t }
  end
end

module Tokenizer
  public
  def Tokenizer.tokenize expression
    line = 1
    tokens = []
    while expression.length > 0
      matched = false
      TOKEN_INFOS.each do |info|
        if (info.first =~ expression) == 0
          matched = true
          tokens << Token.new(info.last, $~.to_s)
          break
        end
      end
      raise UnknownCharError, "Unknown char '#{expression[0]}' in line #{line}" unless matched
      line += 1 if tokens.last.is? :newline
      expression.delete_prefix!(tokens.last.value)
    end
    return TokenList.new tokens
  end
end
