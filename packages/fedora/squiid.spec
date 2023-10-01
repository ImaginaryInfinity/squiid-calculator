

Name:     squiid
Version:  ${VERSION}
Release:  %autorelease
Summary:  Do advanced algebraic and RPN calculations.
License:  GPL-3.0
URL:      https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid
Source:   https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/archive/%{version}/squiid-%{version}.tar.gz
BuildRequires:   make
BuildRequires:   cmake
BuildRequires:   rust

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
%{_bindir}/squiid
%license LICENSE

%changelog
%autochangelog

# %check
# make test
