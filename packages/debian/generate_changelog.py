import requests
from datetime import datetime, timezone, timedelta

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
squiid ({version}-1) focal; urgency=medium

  * {title}
  * View more at {self_url}

 -- ImaginaryInfinity <tabulatejarl8@gmail.com>  {release_time}

"""

print(full_changelog)
