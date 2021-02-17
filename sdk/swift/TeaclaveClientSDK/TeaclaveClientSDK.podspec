Pod::Spec.new do |s|
  s.name = "TeaclaveClientSDK"
  s.version = "0.1.0"
  s.summary = "Teaclave Client SDK."
  s.homepage = "https://teaclave.apache.org"
  s.license = "Apache-2.0"
  s.author = { "Teaclave Contributors" => "dev@teaclave.apache.org" }
  s.ios.deployment_target = '13.0'
  s.source = { :git => "https://github.com/apache/incubator-teaclave.git", :tag => "v0.1.0" }
  s.source_files  = "TeaclaveClietnSDK", "TeaclaveClientSDK/**/*.{h,swift}", "External"
  s.module_map = 'TeaclaveClientSDK/TeaclaveClientSDK.modulemap'
  s.vendored_libraries= 'External/libteaclave_client_sdk.a'
  s.requires_arc = true
  s.static_framework = true
  s.dependency 'OpenSSL-Universal', '~> 1.0.0'
  s.library = 'c++'
end
