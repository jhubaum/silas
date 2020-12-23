require 'fileutils'
require_relative 'org/parser'
require_relative 'ssg/builder'

WEBSITE_INPUT_DIRECTORY = "/home/johannes/gtd/website/"
OUTPUT_FOLDER_NAME = "generated"

if Dir.exist? OUTPUT_FOLDER_NAME
  FileUtils.rm_rf(OUTPUT_FOLDER_NAME)
end
Dir.mkdir OUTPUT_FOLDER_NAME
FileUtils.cp_r("theme/css", OUTPUT_FOLDER_NAME + "/css")

if ARGV[0] == "debug"
  org = OrgFile.new "test/simple.org", nil
  print_element_tree org
  File.open("test.html", "w+") { |f| f.write org.to_html nil }
else
  website = WebsiteBuilder.new WEBSITE_INPUT_DIRECTORY
  website.generate OUTPUT_FOLDER_NAME
end
