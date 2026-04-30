conf = configuration_data()
conf.set_quoted('GETTEXT_PACKAGE', meson.project_name())
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('LOCALEDIR', get_option('prefix') / get_option('localedir'))
conf.set_quoted('OUTPUTDIR', get_option('output-dir'))

config_rs = 'config.rs'
configure_file(input: f'@config_rs@.in', output: config_rs, configuration: conf)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'pfb' / config_rs,
  meson.project_source_root() / 'src' / 'pfb' / config_rs,
  check: true,
)
