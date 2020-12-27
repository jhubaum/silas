require 'fileutils'
require_relative 'org/parser'
require_relative 'ssg/builder'

WEBSITE_INPUT_DIRECTORY = "/home/johannes/gtd/website/"
OUTPUT_FOLDER_NAME = "/home/johannes/projects/silas/generated"

class Config
  def Config.preview
    ARGV[0] == "preview" or target == :debug
  end

  def Config.target
    ARGV[0] == "debug" ? :debug : :website
  end

  def Config.output_directory
    OUTPUT_FOLDER_NAME
  end
end

if Dir.exist? OUTPUT_FOLDER_NAME
  FileUtils.rm_rf(OUTPUT_FOLDER_NAME)
end
Dir.mkdir OUTPUT_FOLDER_NAME
FileUtils.cp_r("theme/css", OUTPUT_FOLDER_NAME + "/css")

if ARGV[0] == "debug"
  #org = OrgFile.new "test/simple.org", nil
  #print_element_tree org
  #File.open("test.html", "w+") { |f| f.write org.to_html nil }
  website = WebsiteBuilder.new "test"
  website.generate OUTPUT_FOLDER_NAME
else
  website = WebsiteBuilder.new WEBSITE_INPUT_DIRECTORY
  website.generate OUTPUT_FOLDER_NAME
end
