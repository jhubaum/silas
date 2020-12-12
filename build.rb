require 'fileutils'
require './orgparse'
require './render'

include SSG

OUTPUT_FOLDER_NAME = "generated"

if Dir.exist? OUTPUT_FOLDER_NAME
  FileUtils.rm_rf(OUTPUT_FOLDER_NAME)
end
Dir.mkdir OUTPUT_FOLDER_NAME
FileUtils.cp_r("css", OUTPUT_FOLDER_NAME + "/css")

f = OrgFile.new ARGV[0]

render_post f, OUTPUT_FOLDER_NAME
