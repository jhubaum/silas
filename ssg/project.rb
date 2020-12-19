class Project
  attr_reader :index, :files

  def initialize path, dirname
    @name = dirname.titlecase
    @url = dirname
    @index, @files = load_files path+dirname
  end

  def url path
    "#{path}/#{@url}"
  end

  def create_link
    Link.new @index, @name
  end

  private
  def load_files path
    files = { }
    index = nil
    Dir.glob("#{path}/*").each do |f|
      puts "#{f}"
      raise "No subfolders allowed" unless File.file? f
      name = f.split("/").last.split(".")
      if name.last != "org"
        puts "Warning: Only org-files allowed. Ignoring #{f}"
      else
        name = name.first
        parsed = OrgParser.parse_file f

        index = parsed if name == "index"
        raise "Duplicate filename #{name}" if files.key? name
        files[name] = parsed
      end
    end
    return index, files
  end
end
