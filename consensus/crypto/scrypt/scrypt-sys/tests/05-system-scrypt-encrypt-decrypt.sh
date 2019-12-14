#!/bin/sh

### Constants
c_valgrind_min=1
reference_file="${scriptdir}/verify-strings/test_scrypt.good"
encrypted_file_1="${out}/sys-scrypt.enc"
decrypted_file_1="${out}/sys-scrypt.txt"
encrypted_file_2="${out}/our-scrypt.enc"
decrypted_file_2="${out}/our-scrypt.txt"

scenario_cmd() {
	if [ -z "${system_scrypt}" ]; then
		printf "no suitable system scrypt: "
		# Inform test suite that we are skipping.
		setup_check_variables
		echo "-1" > ${c_exitfile}
		return
	fi

	# Encrypt a file with our scrypt.
	setup_check_variables
	(
		echo ${password} | ${c_valgrind_cmd} ${bindir}/scrypt	\
		    enc -P -t 1 ${reference_file} ${encrypted_file_1}
		echo $? > ${c_exitfile}
	)

	# Use the system scrypt to decrypt the file we just
	# encrypted. Don't use valgrind for this.
	setup_check_variables
	(
		echo ${password} | ${system_scrypt}			\
		    dec -P ${encrypted_file_1} ${decrypted_file_1}
		echo $? > ${c_exitfile}
	)

	# The decrypted file should match the reference.
	setup_check_variables
	cmp -s ${decrypted_file_1} ${reference_file}
	echo $? > ${c_exitfile}

	# Encrypt a file with the system scrypt.  Don't use
	# valgrind for this.
	setup_check_variables
	(
		echo ${password} | ${system_scrypt}			\
		    enc -P -t 1 ${reference_file} ${encrypted_file_2}
		echo $? > ${c_exitfile}
	)

	# Use our scrypt to decrypt the file we just encrypted.
	setup_check_variables
	(
		echo ${password} | ${c_valgrind_cmd} ${bindir}/scrypt	\
		    dec -P ${encrypted_file_2} ${decrypted_file_2}
		echo $? > ${c_exitfile}
	)

	# The decrypted file should match the reference.
	setup_check_variables
	cmp -s ${decrypted_file_2} ${reference_file}
	echo $? > ${c_exitfile}
}
