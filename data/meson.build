subdir('icons')

systemd_dep = dependency('systemd', required: false)

systemd_user_unit_dir = get_option('systemd_user_unit_dir')
if systemd_user_unit_dir == ''
  if systemd_dep.found()
    systemd_user_unit_dir = systemd_dep.get_variable('systemd_user_unit_dir')
  else
    systemd_user_unit_dir = prefix / libdir / 'systemd' / 'user'
  endif
endif

tmpfiles_dir = get_option('tmpfiles_dir')
if tmpfiles_dir == ''
  if systemd_dep.found()
    tmpfiles_dir = systemd_dep.get_variable('tmpfiles_dir')
  else
    tmpfiles_dir = prefix / libdir / 'tmpfiles.d'
  endif
endif

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test(
    'Validate schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()],
  )
endif
compiled = gnome.compile_schemas(build_by_default: true)

schema = files(f'@app_id@.gschema.xml')
install_data(
  schema,
  install_dir: schemasdir,
)

override_file = '00_mobi.phosh.FirstBoot.gschema.override'
override_data = configuration_data()
override_data.set('bindir', bindir)
configure_file(
  input: f'@override_file@.in',
  output: override_file,
  configuration: override_data,
  install_dir: schemasdir,
)

polkit = '20-phosh-first-boot.rules'
polkit_data = configuration_data()
polkit_data.set('setup_user', get_option('setup-user'))
configure_file(
  input: f'@polkit@.in',
  output: polkit,
  configuration: polkit_data,
  install_dir: polkitdir,
)

import_service = 'phosh-first-boot-importer.service'
import_service_data = configuration_data()
import_service_data.set('libexecdir', libexecdir)
configure_file(
  input: f'@import_service@.in',
  output: import_service,
  configuration: import_service_data,
  install_dir: systemd_user_unit_dir,
)

tmpfile_d = 'phosh-first-boot.conf'
tmpfile_d_data = configuration_data()
tmpfile_d_data.set('dir', get_option('output-dir'))
tmpfile_d_data.set('user', get_option('setup-user'))
tmpfile_d_data.set('group', get_option('output-dir-group'))
configure_file(
  input: f'tmpfiles/@tmpfile_d@.in',
  output: tmpfile_d,
  configuration: tmpfile_d_data,
  install_dir: tmpfiles_dir,
)

desktop_file = f'@app_id@.desktop'
desktop_file_data = configuration_data()
desktop_file_data.set('BINDIR', bindir)
desktop_file_data.set('APP_ID', app_id)
merged = i18n.merge_file(
  input: configure_file(
    input: desktop_file + '.in.in',
    output: desktop_file + '.in',
    configuration: desktop_file_data,
  ),
  output: desktop_file,
  po_dir: '../po',
  install: true,
  install_dir: datadir / 'applications',
  type: 'desktop',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: merged)
endif

install_data(
  'defaults.conf',
  install_dir: defaultsdir,
)

