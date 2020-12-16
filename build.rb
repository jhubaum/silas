require 'fileutils'
require_relative 'org/parser'
require_relative 'render'
require_relative 'ssg/builder'

include SSG

OUTPUT_FOLDER_NAME = "generated"
BLOG_CONTENT_DIRECTORY = "/home/johannes/gtd/website"

if Dir.exist? OUTPUT_FOLDER_NAME
  FileUtils.rm_rf(OUTPUT_FOLDER_NAME)
end
Dir.mkdir OUTPUT_FOLDER_NAME
FileUtils.cp_r("css", OUTPUT_FOLDER_NAME + "/css")

if ARGV[0] == "debug"
  f = OrgFile.new "test/simple.org"
  render_post f, "test.html"
else
  website = WebsiteBuilder.new BLOG_CONTENT_DIRECTORY
  website.generate OUTPUT_FOLDER_NAME
end
