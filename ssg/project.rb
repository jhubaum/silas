require "pathname"
require_relative "../org/types"

class Project < OrgObject
  attr_reader :index, :files, :id

  def initialize dirname, website
    @website = website
    @id = dirname
    @index = IndexOrgFile.new self
    @files = populate_files
  end

  def name
    @id.titlecase
  end

  def url path=nil
    "#{@website.url path}/#{@id}"
  end

  def path
    File.join(@website.path, @id)
  end

  def filename
    @id
  end

  def elements
    [@index] + @files.values.to_a
  end

  def create_link
    Link.new nil, self, name
  end

  def visit visitor
    @index.visit visitor
    @files.values.each { |f| f.visit visitor }
  end

  private
  def populate_files
    pathname = Pathname.new path
    files = {}

    Dir.all_files(path) do |f|
      if f.non_index_org_file?
        relative_filepath = Pathname.new(f).relative_path_from(pathname).to_s
        #puts "File: #{f}; Rel: #{relative_filepath}"
        file = OrgFile.new relative_filepath, self

        raise "duplicate id '#{file.id}' (in project '#{name}')" if files.key? file.id
        files[file.id] = file
      end
    end
    files
  end
end
