require './tokenizer'

class OrgParseError < ::StandardError
end

class OrgReadFileError < OrgParseError
end

class OrgDirectory
  def initialize dirname
    @name = dirname
    @files = []
    raise OrgReadFileError, "Path given for OrgDirectory is no directory" unless Dir.exist? @name

    @files = Dir.glob("**/*.org", base: @name).map { |f| OrgFile.new @name, f }

    @files.each { |f| puts f.path }
  end
end

class OrgFile
  attr_reader :children

  def initialize dirname=nil, file_name
    @name = (dirname == nil) ? file_name : File.join(dirname, file_name)
    raise OrgReadFileError, "file '#{@name}' does not exist" unless File.file? @name

    @preamble = { }
    tokens = Tokenizer.tokenize File.open(@name).read

    #parse preamble
    while tokens.pop_if(:attribute_start) != nil
      val = tokens.pop_expected :word
      tokens.pop_expected :colon
      tokens.pop if tokens.peek? :whitespace
      @preamble[val.to_sym] = tokens_to_s tokens.pop_until(:newline, remove_delim=true)
    end

    while tokens.pop_if(:newline) != nil
    end

    @elements = OrgParsing.parse tokens
  end

  def iterate_elements &block
    @elements.each do |elem|
      block.call elem
      if elem.respond_to? :iterate_elements
        elem.iterate_elements { |e| block.call e }
      end
    end
  end
end

module OrgParsing
  def OrgParsing.parse tokens, until_token=nil
    #puts "Enter Parse (until token: #{until_token})"
    elements = []
    while tokens.has_tokens?
      token = tokens.peek
      return elements if token.is? until_token

      if token.is? :section_start
        section = try_section(tokens)
        if section.level > 0 and elements.last.instance_of? Section
          elements.last.append section
        else
          elements << section
        end
      else
        elements << try_text(tokens)
      end
    end
    elements
  end

  def OrgParsing.try_section tokens
    tokens.pop
    level = 0
    token = tokens.pop
    while token.is? :asterisk
      level += 1
      token = tokens.pop
    end
    raise OrgParseError, "expected whitespace" unless token.is? :whitespace
    title = tokens_to_s tokens.pop_until(:newline)
    tokens.pop
    Section.new level, title, parse(tokens, :section_start)
  end

  def OrgParsing.try_text tokens
    Text.new tokens_to_s(tokens.pop_until :section_start)
  end
end

class Section
  attr_reader :level, :title, :children

  def initialize level, title, children
    @level = level
    @title = title
    @children = children
  end

  def append element
    @children << element
  end

  def heading
    "<h#{@level+2}>#{@title}</h#{@level+2}>"
  end

  def iterate_elements &block
    @children.each do |elem|
      block.call elem
      if elem.respond_to? :iterate_elements
        elem.iterate_elements { |e| block.call e }
      end
    end
  end
end


class Text
  attr_reader :text

  def initialize text
    @text = text
  end
end



#OrgDirectory.new "test"
FILE = OrgFile.new "test/simple.org"
