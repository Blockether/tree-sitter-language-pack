# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'

# Use a separate target dir to avoid lockfile collision with the workspace
ENV['CARGO_TARGET_DIR'] ||= File.expand_path('target', __dir__)

create_rust_makefile('ts_pack_ruby/ts_pack_ruby') do |r|
  r.profile = ENV.fetch('CARGO_PROFILE', 'release').to_sym
  r.ext_dir = File.expand_path('.', __dir__)
end
