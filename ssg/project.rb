class Project
  attr_reader :index, :files

  def initialize orgdir
    @orgdir = orgdir
    @index, @files = load_files

    raise "project #{name} doesn't have an index file" if @index == nil
  end

  def name
    @orgdir.name.titlecase
  end

  def url path
    "#{path}/#{@orgdir.name.snakecase}"
  end

  def create_link
    Link.new @index, @name
  end

  private
  def load_files
    files = { }
    index = nil
    @orgdir.all_files do |filename, file|
      if filename == "index"
        raise "project #{name} redefines index in subdir" unless index == nil
        index = file
      elsif name != "ideas"
        raise "duplicate filename '#{filename}' in project #{name}" if files.key? filename
        files[filename] = file
      end
    end
    return index, files
  end
end
