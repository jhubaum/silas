module Tokenizer
  private
  TOKEN_INFOS = [
    [/#\+/, :attribute_start],
    [/ /, :whitespace],
    [/[[:alpha:]]+/, :word],
    [/:/, :colon],
    [/\n\*/, :section_start],
    [/\*/, :asterisk],
    [/\n/, :newline]
  ]
end

class TokenError < ::StandardError
end

class UnknownCharError < TokenError
end

class TokenListError < TokenError
end

class Token
  attr_reader :type, :value

  def initialize type, value
    @type, @value = type, value
  end

  def is? token_type
    return false if token_type == nil
    @type == token_type
  end

  def to_s
    "<Token #{@type}>"
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

  def peek? token_type
    return false unless has_tokens?
    @tokens.first.is? token_type
  end

  def pop
    result = @tokens[0]
    @tokens = @tokens[1..-1]
    result
  end

  def pop_if token_type
    return nil unless peek? token_type
    pop.value
  end

  def pop_expected token_type
    result = pop
    raise TokenListError, "pop_expected found type #{result.type} but expected #{token_type}" unless result.is? token_type

    result.value
  end

  def pop_until token_type, remove_delim=false
    result = []
    while not peek? token_type and has_tokens?
      result << pop
    end
    if remove_delim
      pop
    end
    result
  end
end

module Tokenizer
  public
  def Tokenizer.tokenize expression
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
      raise UnknownCharError, "Unknown char '#{expression[0]}' in expression" unless matched
      expression.delete_prefix!(tokens.last.value)
    end
    return TokenList.new tokens
  end
end
