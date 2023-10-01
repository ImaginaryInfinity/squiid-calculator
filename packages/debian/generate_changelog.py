import requests
from datetime import datetime, timezone, timedelta
import sys

try:
	distro = sys.argv[1]
except IndexError:
	print("Error: please provide an ubuntu distribution codename")
	sys.exit(1)

response = requests.get("https://gitlab.com/api/v4/projects/44631396/releases").json()

full_changelog = ""

for release in response:
	version = release["tag_name"]
	datetime_object = datetime.strptime(
		release["released_at"], "%Y-%m-%dT%H:%M:%S.%fZ"
	).replace(tzinfo=timezone.utc)
	release_time = datetime_object.astimezone(timezone(timedelta(hours=-4))).strftime(
		"%a, %d %b %Y %H:%M:%S %z",
	)
	title = release["name"]
	self_url = release["_links"]["self"]

	full_changelog += f"""\
squiid ({version}-1-0ubuntu1~{distro}ppa1) {distro}; urgency=medium

  * {title}
  * View more at {self_url}

 -- ImaginaryInfinity <tabulatejarl8@gmail.com>  {release_time}

"""

print(full_changelog)
