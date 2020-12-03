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
  def initialize dirname=nil, file_name
    @name = (dirname == nil) ? file_name : File.join(dirname, file_name)
    raise OrgReadFileError, "file '#{@name}' does not exist" unless File.file? @name

    tokens = Tokenizer.tokenize File.open(@name).read
    puts tokens

  end

  def path
    @name
  end
end

#OrgDirectory.new "test"
OrgFile.new "test/simple.org"
