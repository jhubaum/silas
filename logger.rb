require "colorize"

def load_verbosity
  ARGV.each do |a|
    if /^-(?<v>v+)/ =~ a
      return v.to_s.length
    end
  end
  return 0
end

class Log
  @@verbosity = load_verbosity

  def Log.critical message
    puts ("Critical: ".bold + message).red
  end

  def Log.warning message
    puts ("Warning: ".bold + message).yellow
  end

  def Log.info level, message
    puts "Info: ".bold + message if level <= @@verbosity
  end
end
