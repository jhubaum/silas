require 'fileutils'
require_relative 'org/parser'
require_relative 'ssg/builder'
require_relative 'render'

include SSG

WEBSITE_INPUT_DIRECTORY = "/home/johannes/gtd/website/"
OUTPUT_FOLDER_NAME = "generated"

if Dir.exist? OUTPUT_FOLDER_NAME
  FileUtils.rm_rf(OUTPUT_FOLDER_NAME)
end
Dir.mkdir OUTPUT_FOLDER_NAME
FileUtils.cp_r("theme/css", OUTPUT_FOLDER_NAME + "/css")

if ARGV[0] == "debug"
  f = OrgParser.parse_file "test/simple.org"
  print_element_tree f
  render_post f, OUTPUT_FOLDER_NAME
else
  website = WebsiteBuilder.new WEBSITE_INPUT_DIRECTORY
  website.generate OUTPUT_FOLDER_NAME
end
