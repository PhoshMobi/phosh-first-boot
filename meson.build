project(
  'PhoshFirstBoot',
  'Rust',
  version: '0.1.1',
  meson_version: '>= 1.1',
)

app_id = 'mobi.phosh.FirstBoot'

prefix = get_option('prefix')
datadir = prefix / get_option('datadir')
bindir = prefix / get_option('bindir')
libdir = prefix / get_option('libdir')
libexecdir = prefix / get_option('libexecdir')
schemasdir = datadir / 'glib-2.0' / 'schemas'
polkitdir = datadir / 'polkit-1' / 'rules.d'
defaultsdir = datadir / 'phosh-first-boot'

i18n = import('i18n')
gnome = import('gnome')

subdir('data')
subdir('po')
subdir('src')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
