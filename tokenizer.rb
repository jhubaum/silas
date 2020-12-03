module Tokenizer
  private
  TOKEN_INFOS = [
    [/#\+/, :attribute_start],
    [/ /, :whitespace],
    [/[[:alpha:]]+/, :word],
    [/:/, :colon],
    [/\\n\*/, :section_start],
    [/\*/, :asterisk],
    [/\n/, :newline]
  ]
end

class TokenError < ::StandardError
end

class UnknownCharError < TokenError
end

class Token
  attr_reader :type, :value

  def initialize type, value
    @type, @value = type, value
  end

  def to_s
    "<Token #{@type}>"
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
    return tokens
  end
end
