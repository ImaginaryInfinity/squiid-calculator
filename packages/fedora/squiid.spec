

Name:     squiid
Version:  ${VERSION}
Release:  %autorelease
Summary:  A modular calculator written in Rust.
License:  GPL-3.0
URL:      https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid
Source:   https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/archive/%{version}/squiid-%{version}.tar.gz

%description
Squiid is a modular calculator written in Rust. It is currently very early in
development but it is intended to be the successor to
ImaginaryInfinity Calculator

%prep
%autosetup

%build
%configure
%make_build

%install
%make_install

%files

%changelog
%autochangelog

