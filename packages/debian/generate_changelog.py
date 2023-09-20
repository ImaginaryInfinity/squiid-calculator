import requests
from datetime import datetime

response = requests.get("https://gitlab.com/api/v4/projects/44631396/releases").json()

full_changelog = ''

for release in response:
	version = release['tag_name']
	release_time = datetime.strftime(datetime.strptime(release["released_at"], '%Y-%m-%dT%H:%M:%S.%fZ'), '%a, %d %b %Y %H:%M:%S %z')
	title = release['name']
	self_url = release['_links']['self']

	full_changelog += f'''
squiid ({version}-1) focal; urgency=medium

  * {title}
  See more at {self_url}

 -- ImaginaryInfinity <tabulatejarl8@gmail.com>  {release_time}
'''

print(full_changelog)